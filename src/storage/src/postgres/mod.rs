mod authz_model;
mod helper;
mod tenant;
mod tuple;

use anyhow::Context;
use async_trait::async_trait;
use helper::filter_to_conds;
use sea_orm::*;
use sea_orm::{sea_query::all, DbConn};

use crate::error::StorageError;
use crate::postgres::tuple::ActiveModel;
use crate::{
    AuthzModelReader, AuthzModelWriter, Pagination, RelationshipTupleReader,
    RelationshipTupleWriter, TenantOperator, TupleFilter,
};

#[derive(Debug, Clone)]
pub struct Storage {
    pool: DbConn,
}

#[async_trait]
impl RelationshipTupleReader for Storage {
    async fn list(
        &self,
        tenant_id: &str,
        filter: TupleFilter,
        page: Option<Pagination>,
    ) -> anyhow::Result<(Vec<protocol::Tuple>, u64)> {
        let conds = all![
            tuple::Column::TenantId.eq(tenant_id),
            filter_to_conds(&filter)
        ];
        if let Some(page) = page {
            let query = tuple::Entity::find()
                .filter(conds)
                .paginate(&self.pool, page.size);

            Ok((
                query
                    .fetch_page(page.page)
                    .await?
                    .iter()
                    .map(|t| t.to_owned().into())
                    .collect(),
                query.num_pages().await?,
            ))
        } else {
            let tuples = tuple::Entity::find().filter(conds).all(&self.pool).await?;
            Ok((tuples.iter().map(|t| t.to_owned().into()).collect(), 0))
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
        tuple::Entity::insert_many(tuples).exec(&self.pool).await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: &str, filter: TupleFilter) -> anyhow::Result<()> {
        let conds = all![
            tuple::Column::TenantId.eq(tenant_id),
            filter_to_conds(&filter)
        ];
        tuple::Entity::delete_many()
            .filter(conds)
            .exec(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl AuthzModelReader for Storage {
    async fn get_latest(&self, tenant_id: String) -> anyhow::Result<protocol::AuthzModel> {
        let model = authz_model::Entity::find()
            .filter(authz_model::Column::TenantId.eq(tenant_id))
            .one(&self.pool)
            .await?
            .context(StorageError::NotFoundAuthzModel)?;

        Ok(model.into())
    }

    async fn list(
        &self,
        tenant_id: String,
        page: Option<Pagination>,
    ) -> anyhow::Result<(Vec<protocol::AuthzModel>, u64)> {
        let conds = tuple::Column::TenantId.eq(tenant_id);
        if let Some(page) = page {
            let query = authz_model::Entity::find()
                .filter(conds)
                .paginate(&self.pool, page.size);

            Ok((
                query
                    .fetch_page(page.page)
                    .await?
                    .iter()
                    .map(|t| t.to_owned().into())
                    .collect(),
                query.num_pages().await?,
            ))
        } else {
            let tuples = authz_model::Entity::find()
                .filter(conds)
                .all(&self.pool)
                .await?;
            Ok((tuples.iter().map(|t| t.to_owned().into()).collect(), 0))
        }
    }
}

#[async_trait]
impl AuthzModelWriter for Storage {
    async fn save(&self, tenant_id: String, model: protocol::AuthzModel) -> anyhow::Result<()> {
        let model = authz_model::ActiveModel {
            tenant_id: Set(tenant_id),
            model: Set(model.into()),
            ..Default::default()
        };
        authz_model::Entity::insert(model).exec(&self.pool).await?;
        Ok(())
    }
}

#[async_trait]
impl TenantOperator for Storage {
    async fn create(&self, tenant_id: String, name: String) -> anyhow::Result<()> {
        let model = tenant::ActiveModel {
            id: Set(tenant_id),
            name: Set(name),
            ..Default::default()
        };
        tenant::Entity::insert(model).exec(&self.pool).await?;
        Ok(())
    }

    async fn delete(&self, tenant_id: String) -> anyhow::Result<()> {
        tenant::Entity::delete_many()
            .filter(tenant::Column::Id.eq(tenant_id))
            .exec(&self.pool)
            .await?;
        Ok(())
    }

    async fn get(&self, tenant_id: String) -> anyhow::Result<protocol::Tenant> {
        Ok(tenant::Entity::find_by_id(tenant_id)
            .one(&self.pool)
            .await?
            .context(StorageError::NotFoundTenant)?
            .into())
    }

    async fn list(&self, page: Option<Pagination>) -> anyhow::Result<(Vec<protocol::Tenant>, u64)> {
        if let Some(page) = page {
            let query = tenant::Entity::find().paginate(&self.pool, page.size);

            Ok((
                query
                    .fetch_page(page.page)
                    .await?
                    .iter()
                    .map(|t| t.to_owned().into())
                    .collect(),
                query.num_pages().await?,
            ))
        } else {
            let tuples = tenant::Entity::find().all(&self.pool).await?;
            Ok((tuples.iter().map(|t| t.to_owned().into()).collect(), 0))
        }
    }
}
