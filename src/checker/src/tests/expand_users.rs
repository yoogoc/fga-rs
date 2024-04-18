use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{expander::UsersExpander, tests::init};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct Tuple {
    relation: String,
    object_type: String,
    object_id: String,
    user_type: String,
    user_relation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct ExpandCase {
    tuple: Tuple,
    user_ids: HashSet<String>,
}

#[tokio::test]
async fn expand_users_test() {
    let (model, tuple_reader) = init().await;
    let cases: Vec<ExpandCase> = serde_json::from_str(include_str!("./expand-users.json")).unwrap();

    let users_expander = Arc::new(UsersExpander::new(tuple_reader.clone()));
    for case in cases {
        let model = model.clone();
        assert_eq!(
            users_expander
                .users(
                    model.typesystem.clone(),
                    model.tenant_id,
                    case.tuple.relation.clone(),
                    case.tuple.object_type.clone(),
                    case.tuple.object_id.clone(),
                    case.tuple.user_type.clone(),
                    case.tuple.user_relation.clone(),
                )
                .await
                .unwrap(),
            case.user_ids
        );
    }
}
