use async_trait::async_trait;

use crate::{CheckRequest, CheckResult, Checker};

pub struct RemoteChecker {}

impl RemoteChecker {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Checker for RemoteChecker {
    async fn check(&self, req: CheckRequest) -> anyhow::Result<CheckResult> {
        println!("{:?}", req);
        todo!()
    }

    async fn close(&self) {
        todo!()
    }

    fn name(&self) -> &str {
        "remote"
    }
}
