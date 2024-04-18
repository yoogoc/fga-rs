use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::expander::ObjectsExpander;

use super::init;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct Tuple {
    relation: String,
    object_type: String,
    user_type: String,
    user_id: String,
    user_relation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct ExpandCase {
    tuple: Tuple,
    object_ids: HashSet<String>,
}

#[tokio::test]
async fn expand_objects_test() {
    let (model, tuple_reader) = init().await;
    let cases: Vec<ExpandCase> = serde_json::from_str(include_str!("./expand-objects.json")).unwrap();

    let objects_expander = Arc::new(ObjectsExpander::new(tuple_reader.clone()));
    for case in cases {
        let model = model.clone();
        assert_eq!(
            objects_expander
                .objects(
                    model.typesystem.clone(),
                    model.tenant_id,
                    case.tuple.relation.clone(),
                    case.tuple.object_type.clone(),
                    case.tuple.user_type.clone(),
                    case.tuple.user_id.clone(),
                    case.tuple.user_relation.clone(),
                )
                .await
                .unwrap(),
            case.object_ids
        );
    }
}
