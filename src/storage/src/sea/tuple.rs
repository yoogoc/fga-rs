use chrono::Utc;
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Deserialize, Serialize, Default)]
#[sea_orm(table_name = "relation_tuples")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub tenant_id: String,
    pub user_type: String,
    pub user_id: String,
    pub user_relation: Option<String>,
    pub relation: String,
    pub object_type: String,
    pub object_id: String,
    #[sea_orm(default_expr = "Utc::now().naive_utc()")]
    pub created_at: ChronoDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for protocol::Tuple {
    fn from(t: Model) -> protocol::Tuple {
        protocol::Tuple {
            user_type: t.user_type,
            user_id: t.user_id,
            user_relation: t.user_relation,
            relation: t.relation,
            object_type: t.object_type,
            object_id: t.object_id,
        }
    }
}

impl From<protocol::Tuple> for ActiveModel {
    fn from(t: protocol::Tuple) -> ActiveModel {
        ActiveModel {
            user_type: Set(t.user_type),
            user_id: Set(t.user_id),
            user_relation: Set(t.user_relation),
            relation: Set(t.relation),
            object_type: Set(t.object_type),
            object_id: Set(t.object_id),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        }
    }
}
