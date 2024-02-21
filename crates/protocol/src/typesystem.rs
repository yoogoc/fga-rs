use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Relation, TupleToUserset, Type};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
