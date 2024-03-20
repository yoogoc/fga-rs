use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{error::ModelError, Relation, RelationReference, RelationTypeInfo, Type};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct Typesystem(pub HashMap<String, Type>);

impl Typesystem {
    pub fn get_relation(&self, object_type: &str, relation: &str) -> Result<Relation> {
        let typ = self
            .0
            .get(object_type)
            .ok_or(ModelError::NotFoundRelations(String::from(object_type)))?;
        let rr = typ
            .relations
            .get(relation)
            .context(ModelError::NotFoundRelation(String::from(relation)))?;
        let m = typ
            .metadata
            .get(relation)
            .context(ModelError::NotFoundRelation(String::from(relation)))?;

        Ok(Relation {
            rewrite: rr.to_owned(),
            type_info: RelationTypeInfo {
                directly_related_user_types: m.directly_related_user_types.clone(),
            },
        })
    }

    pub fn get_directly_related_usersets(&self, object_type: &str, relation: &str) -> Result<Vec<RelationReference>> {
        let refs = self.get_directly_related_types(object_type, relation)?;
        Ok(refs
            .iter()
            .filter_map(|rr| match rr {
                RelationReference::Direct(_) => None,
                RelationReference::Relation { .. } => Some(rr.to_owned()),
                RelationReference::Wildcard(_) => Some(rr.to_owned()),
            })
            .collect())
    }
    pub fn get_directly_related_types(&self, object_type: &str, relation: &str) -> Result<Vec<RelationReference>> {
        Ok(self
            .get_relation(object_type, relation)?
            .type_info
            .directly_related_user_types)
    }
}
