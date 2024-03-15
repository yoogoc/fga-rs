use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Tuple {
    pub user_type: String,
    pub user_id: String,
    pub user_relation: Option<String>,
    pub relation: String,
    pub object_type: String,
    pub object_id: String,
    // pub created_at:
}
