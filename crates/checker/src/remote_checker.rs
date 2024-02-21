use crate::{CheckRequest, CheckResult, Checker};

pub struct RemoteChecker {}

impl Checker for RemoteChecker {
    fn check(&self, req: CheckRequest) -> anyhow::Result<CheckResult> {
        println!("{:?}", req);
        todo!()
    }

    fn close(&self) {
        todo!()
    }
}
