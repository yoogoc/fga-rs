use std::collections::HashMap;

use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::{error::ModelError, Relation, RelationReference, TupleToUserset, Type};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Typesystem {
    // [objectType] => typeDefinition
    pub type_definitions: HashMap<String, Type>,
    // [objectType] => [relationName] => relation
    pub relations: HashMap<String, HashMap<String, Relation>>,
    // [objectType] => [relationName] => TTU relation
    pub ttu_relations: HashMap<String, HashMap<String, TupleToUserset>>,

    pub tenant_id: String,
    pub model_id: String,
    pub schema_version: String,
}

impl Typesystem {
    pub fn get_relation(&self, object_type: &str, relation: &str) -> Result<&Relation> {
        self.relations
            .get(object_type)
            .ok_or(ModelError::NotFoundRelations(String::from(object_type)))?
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
