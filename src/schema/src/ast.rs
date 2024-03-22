use std::{collections::HashMap, vec};

use protocol::{RelationMetadata, RelationReference, Type as ProtocolType, Typesystem, Userset};
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
        let mut ts = HashMap::new();
        for typ in self.types {
            let mut relations = HashMap::new();
            let mut metadata = HashMap::new();

            for rel in &typ.relations {
                let mut directly_related_user_types = vec![];
                for sub in &rel.subjects {
                    match sub {
                        RelationshipSet::Single(ref user) => {
                            relations.insert(String::from(&rel.name), Userset::This);
                            directly_related_user_types.push(RelationReference::Direct(String::from(user)));
                        }
                        RelationshipSet::Set(ref user, ref relation) => {
                            relations.insert(String::from(&rel.name), Userset::This);
                            if relation.eq("*") {
                                directly_related_user_types.push(RelationReference::Wildcard(String::from(user)));
                            } else {
                                directly_related_user_types.push(RelationReference::Relation {
                                    r#type: String::from(user),
                                    relation: String::from(relation),
                                });
                            }
                        }
                    }
                }
                metadata.insert(
                    String::from(&rel.name),
                    RelationMetadata {
                        directly_related_user_types,
                    },
                );
            }

            let t = ProtocolType {
                name: String::from(&typ.name),
                relations,
                metadata,
            };

            ts.insert(typ.name, t);
        }

        Typesystem(ts)
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
#[serde(rename_all = "snake_case")]
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

impl Relationship {
    pub fn compute(self) -> Relationship {
        todo!()
    }
}

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipSet {
    Single(String),
    Set(String, String),
}
