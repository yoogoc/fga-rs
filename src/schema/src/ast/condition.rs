use schemars::JsonSchema;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub use evalexpr::{Node, Operator, Value};

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Condition {
    pub name: String,
    pub args: Vec<ConditionArg>,
    // pub body: ConditionExpression,
}

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct ConditionArg {
    pub name: String,
    pub r#type: ConditionType,
}

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub enum ConditionType {
    Int,
    Uint,
    Double,
    Bool,
    Bytes,
    String,
    Duration,
    Timestamp,
    Any,
    List(Box<ConditionType>),
    Map(Box<ConditionType>),
    IPaddress,
}
