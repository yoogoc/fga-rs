mod error;
pub mod postgres;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use protocol::{Tenant, Tuple};
use schema::Schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    pub size: u64,
    pub page: u64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
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
pub trait RelationshipTupleReader: Send + Sync {
    async fn list(
        &self,
        tenant_id: &str,
        filter: TupleFilter,
        page: Option<Pagination>,
    ) -> Result<(Vec<Tuple>, Option<u64>)>;
}

#[async_trait]
pub trait RelationshipTupleWriter: Send + Sync {
    async fn save(&self, tenant_id: &str, tuples: Vec<Tuple>) -> Result<()>;
    async fn delete(&self, tenant_id: &str, filter: TupleFilter) -> Result<()>;
}

#[async_trait]
pub trait AuthzModelReader: Send + Sync {
    async fn get_latest(&self, tenant_id: String) -> Result<Schema>;
    async fn list(&self, tenant_id: String, page: Option<Pagination>) -> Result<(Vec<Schema>, Option<u64>)>;
}

#[async_trait]
pub trait AuthzModelWriter: Send + Sync {
    async fn save(&self, tenant_id: String, model: Schema) -> Result<()>;
}

#[async_trait]
pub trait TenantOperator: Send + Sync {
    async fn create(&self, tenant_id: String, name: String) -> Result<()>;
    async fn delete(&self, tenant_id: String) -> Result<()>;
    async fn get(&self, tenant_id: String) -> Result<Tenant>;
    async fn list(&self, page: Option<Pagination>) -> Result<(Vec<Tenant>, Option<u64>)>;
}

pub type RelationshipTupleReaderRef = Arc<dyn RelationshipTupleReader>;
pub type RelationshipTupleWriterRef = Arc<dyn RelationshipTupleWriter>;
pub type AuthzModelReaderRef = Arc<dyn AuthzModelReader>;
pub type AuthzModelWriterRef = Arc<dyn AuthzModelWriter>;
pub type TenantOperatorRef = Arc<dyn TenantOperator>;
