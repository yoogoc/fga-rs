pub mod config;
mod error;
mod grpc;
mod http;

use anyhow::Result;
use checker::expander::{Expander, ObjectsExpander, UsersExpander};
use config::Config;
use http::HttpServer;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, sync::Arc};
use storage::sea;

use crate::grpc::GrpcServer;

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate async_trait;

#[async_trait]
pub trait Server: Send + Sync {
    async fn shutdown(&self) -> Result<()>;
    async fn start(&self, listening: SocketAddr) -> Result<SocketAddr>;
    fn name(&self) -> &str;
}

pub struct Servers {
    // config: Config,
    servers: Vec<(Box<dyn Server>, SocketAddr)>,
}

impl Servers {
    pub async fn new(config: Config) -> Self {
        info!("init servers");
        let mut options = ConnectOptions::new(&config.datasource.uri);
        options.set_schema_search_path(&config.datasource.schema);
        options.sqlx_logging_level(log::LevelFilter::Debug);
        let conn = Database::connect(options).await.unwrap();
        let storage = Arc::new(sea::Storage::new(Arc::new(conn)));
        let tuple_reader = storage.clone();
        let tuple_writer = storage.clone();
        let authz_model_reader = storage.clone();
        let authz_model_writer = storage.clone();
        let tenant_operator = storage.clone();

        let expander = Arc::new(Expander::new(tuple_reader.clone()));
        let objects_expander = Arc::new(ObjectsExpander::new(tuple_reader.clone()));
        let users_expander = Arc::new(UsersExpander::new(tuple_reader.clone()));

        // config distributed: if distributed { remote } else { local }
        // let resolver = Arc::new(checker::RemoteChecker::new());
        let local_checker = Arc::new(checker::LocalChecker::new(None, storage.clone()));
        let cache_checker = Arc::new(checker::CacheChecker::new(local_checker.clone()));

        let mut servers = Vec::<(Box<dyn Server>, SocketAddr)>::with_capacity(2);
        if let Some(http) = &config.http {
            let server = HttpServer::new(
                tuple_reader,
                tuple_writer,
                authz_model_reader.clone(),
                authz_model_writer,
                tenant_operator,
                cache_checker.clone(),
                expander,
                objects_expander,
                users_expander,
            );
            servers.push((Box::new(server), http.addr.parse::<SocketAddr>().unwrap()));
        }
        if let Some(grpc) = &config.grpc {
            let server = GrpcServer::new(cache_checker.clone(), authz_model_reader.clone());
            servers.push((Box::new(server), grpc.addr.parse::<SocketAddr>().unwrap()));
        }

        Self { servers }
    }

    pub async fn start(&self) -> Result<()> {
        futures::future::try_join_all(self.servers.iter().map(start_server))
            .await
            .map(|_| ())
    }

    pub async fn shutdown(&self) -> Result<()> {
        futures::future::try_join_all(self.servers.iter().map(|s| s.0.shutdown()))
            .await
            .map(|_| ())
    }
}

pub async fn start_server(server_and_addr: &(Box<dyn Server>, SocketAddr)) -> Result<Option<SocketAddr>> {
    let (server, addr) = server_and_addr;
    info!("Start {} at {}", server.name(), addr);
    server.start(*addr).await.map(Some)
}
