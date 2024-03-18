use std::vec;

use protocol::Typesystem;
use schemars::JsonSchema;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, Default, JsonSchema)]
pub struct Schema {
    pub types: Vec<Type>,
    // pub conds: Vec<Condition>,
}

impl Schema {
    pub fn new(types: Vec<SchemaUnit>) -> Self {
        let mut ts = vec![];
        for typ in types {
            match typ {
                SchemaUnit::Type(typ) => ts.push(typ),
            }
        }
        Self { types: ts }
    }

    pub fn to_typesystem(self) -> Typesystem {
        // let mut ts = HashMap::new();
        // for typ in self.types {
        //     ts.insert(typ.name, typ.into());
        // }

        // Typesystem(ts)
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum SchemaUnit {
    Type(Type),
}

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Type {
    pub name: String,
    pub relations: Vec<Relation>,
    pub permissions: Vec<Permission>,
}

impl Type {
    pub fn new(name: String, rops: Vec<RelationOrPermission>) -> Self {
        let mut relations = vec![];
        let mut permissions = vec![];
        for rop in rops {
            match rop {
                RelationOrPermission::Relation(r) => relations.push(r),
                RelationOrPermission::Permission(p) => permissions.push(p),
            }
        }
        Self {
            name,
            relations,
            permissions,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RelationOrPermission {
    Relation(Relation),
    Permission(Permission),
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Relation {
    pub name: String,
    pub subjects: Vec<RelationshipSet>,
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub struct Permission {
    pub name: String,
    pub permissions: Vec<Relationship>,
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub enum Relationship {
    Set(RelationshipSet),
    Union {
        children: Vec<Box<Relationship>>,
    },
    Intersection {
        children: Vec<Box<Relationship>>,
    },
    Difference {
        base: Box<Relationship>,
        subtract: Box<Relationship>,
    },
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
pub enum RelationshipSet {
    Single(String),
    Set(String, String),
}
