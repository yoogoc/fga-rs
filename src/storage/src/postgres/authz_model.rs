use chrono::Utc;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Deserialize, Serialize, Default)]
#[sea_orm(table_name = "authz_models")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub tenant_id: String,
    pub model: Json,
    #[sea_orm(default_expr = "Utc::now().naive_utc()")]
    pub created_at: ChronoDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for protocol::AuthzModel {
    // TODO
    fn from(_t: Model) -> protocol::AuthzModel {
        protocol::AuthzModel { types: vec![] }
    }
}
