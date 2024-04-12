mod zanzibar;

use std::net::SocketAddr;

use crate::error::ServerError;
use crate::Server;
use anyhow::{ensure, Result};
use async_trait::async_trait;
use checker::CheckerRef;
use futures::FutureExt;
use proto::fgars_service_server::FgarsServiceServer;
use storage::AuthzModelReaderRef;
use tokio::sync::oneshot::{self, Sender};
use tokio::sync::Mutex;
use tonic::transport::Server as TonicServer;
use tonic_reflection::server::Builder as ReflectionBuilder;

pub struct GrpcServer {
    checker: CheckerRef,
    model_reader: AuthzModelReaderRef,
    shutdown_tx: Mutex<Option<Sender<()>>>,
}

impl GrpcServer {
    pub fn new(checker: CheckerRef, model_reader: AuthzModelReaderRef) -> Self {
        Self {
            checker,
            model_reader,
            shutdown_tx: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Server for GrpcServer {
    async fn shutdown(&self) -> Result<()> {
        let mut shutdown_tx = self.shutdown_tx.lock().await;
        if let Some(tx) = shutdown_tx.take() {
            if tx.send(()).is_err() {
                // info!("Receiver dropped, the grpc server has already existed");
            }
        }
        Ok(())
    }
    async fn start(&self, listening: SocketAddr) -> Result<SocketAddr> {
        let (tx, rx) = oneshot::channel();

        let server = {
            let mut shutdown_tx = self.shutdown_tx.lock().await;
            ensure!(
                shutdown_tx.is_none(),
                ServerError::AlreadyStarted { server: "GRPC".into() }
            );

            // server refection
            let service = ReflectionBuilder::configure()
                .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
                .build()
                .unwrap();

            let server = TonicServer::builder()
                .add_service(service)
                .add_service(FgarsServiceServer::new(zanzibar::Service {
                    checker: self.checker.clone(),
                    model_reader: self.model_reader.clone(),
                }))
                .serve_with_shutdown(listening, rx.map(drop));
            *shutdown_tx = Some(tx);

            server
        };

        server.await?;
        Ok(listening)
    }
    fn name(&self) -> &str {
        "grpc"
    }
}
