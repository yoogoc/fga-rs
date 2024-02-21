mod tuple;
mod typesystem;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub use tuple::Tuple;
pub use typesystem::Typesystem;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AuthzModel {
    pub types: Vec<Type>,
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
    Union {
        children: Vec<Userset>,
    },
    Intersection {
        children: Vec<Userset>,
    },
    Difference {
        base: Box<Userset>,
        subtract: Box<Userset>,
    },
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TupleKey {
    pub object: String,
    pub relation: String,
    pub user: String,
}

impl TupleKey {
    pub fn cache_key(&self) -> String {
        format!("{}-{}-{}", &self.object, &self.relation, &self.user,)
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
