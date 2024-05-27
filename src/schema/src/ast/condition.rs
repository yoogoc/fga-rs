use schemars::JsonSchema;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub use evalexpr::{Node, Operator, Value};

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Condition {
    pub name: String,
    pub args: Vec<ConditionArg>,
    pub body: Vec<ConditionExpression>,
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

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub enum ConditionExpression {
    Identifier(String),    // identifier
    Colon,                 // :
    LBracket,              // (
    RBracket,              // )
    Caret,                 // ^
    And,                   // &
    Or,                    // |
    Add,                   // +
    Sub,                   // -
    YulArrow,              // ->
    Sharp,                 // #
    Newline,               // \n
    LBrace,                // {
    RBrace,                // }
    Star,                  // \*
    Slash,                 // /
    Percent,               // %
    Comma,                 // ,
    Semicolon,             // ;
    Eq,                    // =
    ExclamationMark,       // (nothing)!
    Gt,                    // >
    Lt,                    // <
    Condition,             // condition
    Int,                   // int
    Uint,                  // uint
    Double,                // double
    Bool,                  // bool
    Bytes,                 // bytes
    String,                // string
    Duration,              // duration
    Timestamp,             // timestamp
    Any,                   // any
    List,                  // list
    Map,                   // map
    IPaddress,             // ipaddress
    StringLiteral(String), // string literal
    IntLiteral(i64),       // int literal
    DoubleLiteral(f64),    // double literal
    Dollar,                // $
    GraveAccent,           // `
    WhiteSpace,            // space
    Point,                 // .
}
