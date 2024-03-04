use std::net::SocketAddr;

use crate::Server;
use anyhow::Result;
use async_trait::async_trait;

pub struct GrpcServer {}

#[async_trait]
impl Server for GrpcServer {
    async fn shutdown(&self) -> Result<()> {
        todo!()
    }
    async fn start(&self, listening: SocketAddr) -> Result<SocketAddr> {
        todo!()
    }
    fn name(&self) -> &str {
        todo!()
    }
}
