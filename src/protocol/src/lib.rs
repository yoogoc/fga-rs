mod error;
mod tuple;
mod typesystem;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub use tuple::Tuple;
pub use typesystem::Typesystem;

pub const WILDCARD: &str = "*";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AuthzModel {
    pub types: Vec<Type>,
}

impl Into<serde_json::Value> for AuthzModel {
    fn into(self) -> serde_json::Value {
        // TODO
        serde_json::Value::Null
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Type {
    pub name: String,
    pub relations: HashMap<String, Userset>,
    pub metadata: HashMap<String, RelationMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Userset {
    This,
    Computed(ObjectRelation),
    TupleTo(TupleToUserset),
    Union { children: Vec<Box<Userset>> },
    Intersection { children: Vec<Box<Userset>> },
    Difference { base: Box<Userset>, subtract: Box<Userset> },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SetOperator {
    Union,
    Intersection,
    Exclusion,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TupleToUserset {
    pub tupleset: ObjectRelation,
    pub computed_userset: ObjectRelation,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ObjectRelation {
    pub object: String,
    pub relation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RelationMetadata {
    pub directly_related_user_types: Vec<RelationReference>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RelationReference {
    Direct(String),
    Relation { r#type: String, relation: String },
    Wildcard(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TupleKey {
    pub user_type: String,
    pub user_id: String,
    pub user_relation: String,
    pub relation: String,
    pub object_type: String,
    pub object_id: String,
}

impl TupleKey {
    pub fn cache_key(&self) -> String {
        format!(
            "{}{}-{}-{}{}{}",
            &self.object_type, &self.object_id, &self.relation, &self.user_type, &self.user_id, &self.user_relation
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Relation {
    pub name: String,
    pub rewrite: Userset,
    pub type_info: RelationTypeInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RelationTypeInfo {
    pub directly_related_user_types: Vec<RelationReference>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tenant {
    pub id: String,
    pub name: String,
}
