mod condition;
mod r#type;

use std::{collections::HashMap, vec};

pub use condition::*;
use protocol::{RelationMetadata, RelationReference, Type as ProtocolType, Typesystem, Userset};
pub use r#type::*;
use schemars::JsonSchema;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, FromJsonQueryResult, Deserialize, Serialize, Default, JsonSchema)]
pub struct Schema {
    pub types: Vec<Type>,
    pub conditions: Vec<Condition>,
}

impl Schema {
    pub fn new(units: Vec<SchemaUnit>) -> Self {
        let mut types = vec![];
        let mut conditions = vec![];
        for unit in units {
            match unit {
                SchemaUnit::Type(typ) => types.push(typ),
                SchemaUnit::Condition(cond) => conditions.push(cond),
            }
        }
        Self { types, conditions }
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

            for permission in typ.permissions {
                relations.insert(String::from(&permission.name), permission.permission.to_userset());
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
    Condition(Condition),
}
