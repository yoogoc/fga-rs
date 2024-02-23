mod error;
pub mod postgres;

use anyhow::Result;
use async_trait::async_trait;
use protocol::{AuthzModel, Tenant, Tuple};

pub struct Pagination {
    pub size: u64,
    pub page: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TupleFilter {
    pub object_type_eq: Option<String>,
    pub object_id_eq: Option<String>,
    pub object_id_in: Option<Vec<String>>,
    pub relation_eq: Option<String>,
    pub user_type_eq: Option<String>,
    pub user_id_eq: Option<String>,
    pub user_id_in: Option<Vec<String>>,
    pub user_relation_eq: Option<String>,
    pub or: Option<Vec<TupleFilter>>,
}

#[async_trait]
pub trait RelationshipTupleReader {
    async fn list(
        &self,
        tenant_id: &str,
        filter: TupleFilter,
        page: Option<Pagination>,
    ) -> Result<(Vec<Tuple>, u64)>;
}

#[async_trait]
pub trait RelationshipTupleWriter {
    async fn save(&self, tenant_id: &str, tuples: Vec<Tuple>) -> Result<()>;
    async fn delete(&self, tenant_id: &str, filter: TupleFilter) -> Result<()>;
}

#[async_trait]
pub trait AuthzModelReader {
    async fn get_latest(&self, tenant_id: String) -> Result<AuthzModel>;
    async fn list(
        &self,
        tenant_id: String,
        page: Option<Pagination>,
    ) -> Result<(Vec<AuthzModel>, u64)>;
}

#[async_trait]
pub trait AuthzModelWriter {
    async fn save(&self, tenant_id: String, model: AuthzModel) -> Result<()>;
}

#[async_trait]
pub trait TenantOperator {
    async fn create(&self, tenant_id: String, name: String) -> Result<()>;
    async fn delete(&self, tenant_id: String) -> Result<()>;
    async fn get(&self, tenant_id: String) -> Result<Tenant>;
    async fn list(&self, page: Option<Pagination>) -> Result<(Vec<Tenant>, u64)>;
}
