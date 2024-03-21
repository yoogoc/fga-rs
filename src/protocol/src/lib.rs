mod error;
mod tuple;
mod typesystem;

use std::{collections::HashMap, fmt};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use tuple::Tuple;
pub use typesystem::*;

pub const WILDCARD: &str = "*";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Type {
    pub name: String,
    pub relations: HashMap<String, Userset>,
    pub metadata: HashMap<String, RelationMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct TupleToUserset {
    pub tupleset: ObjectRelation,
    pub computed_userset: ObjectRelation,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ObjectRelation {
    pub object: String,
    pub relation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct RelationMetadata {
    pub directly_related_user_types: Vec<RelationReference>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub enum RelationReference {
    Direct(String),
    Relation { r#type: String, relation: String },
    Wildcard(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
pub struct TupleKey {
    pub user_type: String,
    pub user_id: String,
    pub user_relation: String,
    pub relation: String,
    pub object_type: String,
    pub object_id: String,
}

impl fmt::Display for TupleKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.user_relation.eq("") {
            write!(
                f,
                "{}:{}-{}-{}:{}",
                &self.object_type, &self.object_id, &self.relation, &self.user_type, &self.user_id
            )
        } else {
            write!(
                f,
                "{}:{}-{}-{}:{}#{}",
                &self.object_type, &self.object_id, &self.relation, &self.user_type, &self.user_id, &self.user_relation
            )
        }
    }
}

impl TupleKey {
    pub fn cache_key(&self) -> String {
        format!(
            "{}{}-{}-{}{}{}",
            &self.object_type, &self.object_id, &self.relation, &self.user_type, &self.user_id, &self.user_relation
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Relation {
    // pub name: String,
    pub rewrite: Userset,
    pub type_info: RelationTypeInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct RelationTypeInfo {
    pub directly_related_user_types: Vec<RelationReference>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Tenant {
    pub id: String,
    pub name: String,
}
