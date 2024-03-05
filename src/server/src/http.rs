use std::{net::SocketAddr, time::Duration};

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use std::sync::Mutex as StdMutex;
use tokio::sync::oneshot::{self, Sender};

use crate::error::ServerError;
use crate::Server;
use anyhow::{ensure, Result};
use async_trait::async_trait;
use axum::Router;
use tokio::sync::Mutex;

pub struct HttpServer {
    router: StdMutex<Router>,
    shutdown_tx: Mutex<Option<Sender<()>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpOptions {
    pub addr: String,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:4000".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl HttpServer {
    fn new(options: HttpOptions) -> Self {
        let router = Router::new();
        Self {
            router: StdMutex::new(router),
            shutdown_tx: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Server for HttpServer {
    async fn shutdown(&self) -> Result<()> {
        let mut shutdown_tx = self.shutdown_tx.lock().await;
        if let Some(tx) = shutdown_tx.take() {
            if tx.send(()).is_err() {
                // info!("Receiver dropped, the HTTP server has already existed");
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
                ServerError::AlreadyStarted { server: "HTTP".into() }
            );

            let listener = tokio::net::TcpListener::bind(listening).await.unwrap();

            let server = axum::serve(listener, self.router.lock().unwrap().clone().into_make_service())
                .with_graceful_shutdown(rx.map(drop));

            *shutdown_tx = Some(tx);

            server
        };

        tokio::spawn(async move { server.await });

        Ok(listening)
    }
    fn name(&self) -> &str {
        "http"
    }
}
