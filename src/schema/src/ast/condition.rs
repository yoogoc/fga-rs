use schemars::JsonSchema;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub use evalexpr::{Node, Operator, Value};

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Condition {
    pub name: String,
    pub args: Vec<Arg>,
    // pub body: Node,
}

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Arg {
    pub name: String,
    pub r#type: ArgType,
}

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub enum ArgType {
    String,
}
