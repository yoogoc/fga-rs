pub mod authz_model;
mod helper;
mod tenant;
pub mod tuple;

use std::sync::Arc;

use ::schema::Schema as AuthzModel;
use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use helper::filter_to_conds;
use sea_orm::*;
use sea_orm::{sea_query::all, DbConn};

use crate::error::StorageError;
use crate::sea::tuple::ActiveModel;
use crate::{
    AuthzModelReader, AuthzModelWriter, Pagination, RelationshipTupleReader, RelationshipTupleWriter, TenantOperator,
    TupleFilter,
};

#[derive(Debug, Clone)]
pub struct Storage {
    pool: Arc<DbConn>,
}

impl Storage {
    pub fn new(pool: Arc<DbConn>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RelationshipTupleReader for Storage {
    async fn list(
        &self,
        tenant_id: &str,
        filter: TupleFilter,
        page: Option<Pagination>,
    ) -> anyhow::Result<(Vec<protocol::Tuple>, Option<u64>)> {
        let conds = all![tuple::Column::TenantId.eq(tenant_id), filter_to_conds(&filter)];
        let conn = self.pool.clone();
        if let Some(page) = page {
            let query = tuple::Entity::find().filter(conds).paginate(conn.as_ref(), page.size);

            Ok((
                query
                    .fetch_page(page.page - 1)
                    .await?
                    .iter()
                    .map(|t| t.to_owned().into())
                    .collect(),
                Some(query.num_pages().await?),
            ))
        } else {
            let tuples = tuple::Entity::find().filter(conds).all(conn.as_ref()).await?;
            Ok((tuples.iter().map(|t| t.to_owned().into()).collect(), None))
        }
    }
}

#[async_trait]
impl RelationshipTupleWriter for Storage {
    async fn save(&self, tenant_id: &str, tuples: Vec<protocol::Tuple>) -> anyhow::Result<()> {
        let mut tuples: Vec<ActiveModel> = tuples.iter().map(|t| t.to_owned().into()).collect();
        for t in &mut tuples {
            t.tenant_id = Set(tenant_id.to_owned());
        }
        tuple::Entity::insert_many(tuples)
            .exec(self.pool.clone().as_ref())
            .await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: &str, filter: TupleFilter) -> anyhow::Result<()> {
        let conds = all![tuple::Column::TenantId.eq(tenant_id), filter_to_conds(&filter)];
        tuple::Entity::delete_many()
            .filter(conds)
            .exec(self.pool.clone().as_ref())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl AuthzModelReader for Storage {
    async fn get_latest(&self, tenant_id: String) -> anyhow::Result<(String, AuthzModel)> {
        let model = authz_model::Entity::find()
            .filter(authz_model::Column::TenantId.eq(tenant_id))
            .one(self.pool.clone().as_ref())
            .await?
            .context(StorageError::NotFoundAuthzModel)?;

        Ok((model.id.to_string(), model.model))
    }

    async fn get(&self, tenant_id: String, id: String) -> anyhow::Result<(String, AuthzModel)> {
        let model = authz_model::Entity::find()
            .filter(authz_model::Column::TenantId.eq(tenant_id))
            .filter(authz_model::Column::Id.eq(id))
            .one(self.pool.clone().as_ref())
            .await?
            .context(StorageError::NotFoundAuthzModel)?;

        Ok((model.id.to_string(), model.model))
    }

    async fn list(
        &self,
        tenant_id: String,
        page: Option<Pagination>,
    ) -> anyhow::Result<(Vec<(String, AuthzModel)>, Option<u64>)> {
        let conds = authz_model::Column::TenantId.eq(tenant_id);
        let conn = self.pool.clone();
        if let Some(page) = page {
            let query = authz_model::Entity::find()
                .filter(conds)
                .paginate(conn.as_ref(), page.size);

            Ok((
                query
                    .fetch_page(page.page - 1)
                    .await?
                    .iter()
                    .map(|t| (t.id.to_string(), t.model.to_owned()))
                    .collect(),
                Some(query.num_pages().await?),
            ))
        } else {
            let list = authz_model::Entity::find().filter(conds).all(conn.as_ref()).await?;
            Ok((
                list.iter().map(|t| (t.id.to_string(), t.model.to_owned())).collect(),
                None,
            ))
        }
    }
}

#[async_trait]
impl AuthzModelWriter for Storage {
    async fn save(&self, tenant_id: String, model: AuthzModel) -> anyhow::Result<()> {
        let model = authz_model::ActiveModel {
            tenant_id: Set(tenant_id),
            model: Set(model),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        authz_model::Entity::insert(model)
            .exec(self.pool.clone().as_ref())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TenantOperator for Storage {
    async fn create(&self, tenant_id: String, name: String) -> anyhow::Result<()> {
        let model = tenant::ActiveModel {
            id: Set(tenant_id),
            name: Set(name),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        tenant::Entity::insert(model).exec(self.pool.clone().as_ref()).await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: String) -> anyhow::Result<()> {
        tenant::Entity::delete_many()
            .filter(tenant::Column::Id.eq(tenant_id))
            .exec(self.pool.clone().as_ref())
            .await?;
        Ok(())
    }

    async fn get(&self, tenant_id: String) -> anyhow::Result<protocol::Tenant> {
        Ok(tenant::Entity::find_by_id(tenant_id)
            .one(self.pool.clone().as_ref())
            .await?
            .context(StorageError::NotFoundTenant)?
            .into())
    }

    async fn list(&self, page: Option<Pagination>) -> anyhow::Result<(Vec<protocol::Tenant>, Option<u64>)> {
        let conn = self.pool.clone();
        if let Some(page) = page {
            let query = tenant::Entity::find().paginate(conn.as_ref(), page.size);

            Ok((
                query
                    .fetch_page(page.page - 1)
                    .await?
                    .iter()
                    .map(|t| t.to_owned().into())
                    .collect(),
                Some(query.num_pages().await?),
            ))
        } else {
            let tuples = tenant::Entity::find().all(conn.as_ref()).await?;
            Ok((tuples.iter().map(|t| t.to_owned().into()).collect(), None))
        }
    }
}
