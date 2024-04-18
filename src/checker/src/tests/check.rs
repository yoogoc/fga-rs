use std::sync::Arc;

use protocol::TupleKey;
use serde::{Deserialize, Serialize};

use crate::{CheckRequest, CheckerRef, LocalChecker};

use super::init;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct CheckCase {
    tuple: TupleKey,
    allow: bool,
}

#[tokio::test]
async fn check_test() {
    let (model, tuple_reader) = init().await;
    let cases: Vec<CheckCase> = serde_json::from_str(include_str!("./check-cases.json")).unwrap();

    let local_checker: CheckerRef = Arc::new(LocalChecker::new(None, tuple_reader.clone()));
    for case in cases {
        let model = model.clone();
        let req = CheckRequest {
            tenant_id: model.tenant_id.clone(),
            model_id: model.tenant_id.clone(),
            typesystem: model.typesystem.clone(),
            tuple_key: case.tuple,
            ..Default::default()
        };
        assert_eq!(local_checker.check(req).await.unwrap_or_default().allow, case.allow);
    }
}
