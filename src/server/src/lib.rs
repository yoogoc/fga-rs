mod error;
mod grpc;
mod http;

use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;

#[async_trait]
pub trait Server: Send + Sync {
    async fn shutdown(&self) -> Result<()>;
    async fn start(&self, listening: SocketAddr) -> Result<SocketAddr>;
    fn name(&self) -> &str;
}

pub struct Servers {
    servers: Vec<(Box<dyn Server>, SocketAddr)>,
}

impl Servers {
    pub fn new() -> Self {
        todo!()
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
    // info!("Starting {} at {}", server.name(), addr);
    server.start(*addr).await.map(Some)
}
