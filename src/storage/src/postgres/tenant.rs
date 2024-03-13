use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Deserialize, Serialize, Default)]
#[sea_orm(table_name = "tenants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub created_at: ChronoDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for protocol::Tenant {
    fn from(t: Model) -> protocol::Tenant {
        protocol::Tenant { id: t.id, name: t.name }
    }
}
