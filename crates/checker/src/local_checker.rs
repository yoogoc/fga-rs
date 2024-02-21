use crate::{CheckRequest, CheckResult, Checker};

pub struct LocalChecker {
    pub resolver: Box<dyn Checker>,
}

impl Checker for LocalChecker {
    fn check(&self, req: CheckRequest) -> anyhow::Result<CheckResult> {
        println!("{:?}", req);
        todo!()
    }

    fn close(&self) {
        todo!()
    }
}
