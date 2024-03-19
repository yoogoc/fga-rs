use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{error::ModelError, Relation, RelationReference, TupleToUserset, Type};

pub type TypeSet = (Type, HashMap<String, Relation>, HashMap<String, TupleToUserset>);
// [objectType] => (typeDefinition, [relationName] => relation, [relationName] => TTU relation)
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct Typesystem(pub HashMap<String, TypeSet>);

impl Typesystem {
    pub fn get_relation(&self, object_type: &str, relation: &str) -> Result<&Relation> {
        self.0
            .get(object_type)
            .ok_or(ModelError::NotFoundRelations(String::from(object_type)))?
            .1
            .get(relation)
            .context(ModelError::NotFoundRelation(String::from(relation)))
    }

    pub fn get_directly_related_usersets(&self, object_type: &str, relation: &str) -> Result<Vec<&RelationReference>> {
        let refs = self.get_directly_related_types(object_type, relation)?;
        Ok(refs
            .iter()
            .filter_map(|rr| match rr {
                RelationReference::Direct(_) => None,
                RelationReference::Relation { .. } => Some(rr),
                RelationReference::Wildcard(_) => Some(rr),
            })
            .collect())
    }
    pub fn get_directly_related_types(&self, object_type: &str, relation: &str) -> Result<&Vec<RelationReference>> {
        Ok(&self
            .get_relation(object_type, relation)?
            .type_info
            .directly_related_user_types)
    }
}
