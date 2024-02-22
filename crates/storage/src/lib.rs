use anyhow::Result;
use protocol::{AuthzModel, Tenant, Tuple};

pub struct Pagination {
    pub size: u32,
    pub page: u32,
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

pub trait RelationshipTupleReader {
    fn list(
        &self,
        tenant_id: &str,
        filter: TupleFilter,
        page: Option<Pagination>,
    ) -> Result<(Vec<Tuple>, u32)>;
}

pub trait RelationshipTupleWriter {
    fn save(&self, tenant_id: &str, tuples: Vec<Tuple>) -> Result<()>;
    fn delete(&self, tenant_id: &str, filter: TupleFilter) -> Result<()>;
}

pub trait AuthzModelReader {
    fn get_latest(&self, tenant_id: String) -> Result<AuthzModel>;
    fn list(&self, tenant_id: String, page: Option<Pagination>) -> Result<(Vec<AuthzModel>, u32)>;
}

pub trait AuthzModelWriter {
    fn save(&self, tenant_id: String, model: AuthzModel) -> Result<()>;
}

pub trait TenantOperator {
    fn create(&self, tenant_id: String, name: String) -> Result<()>;
    fn delete(&self, tenant_id: String) -> Result<()>;
    fn get(&self, tenant_id: String) -> Result<Tenant>;
    fn list(&self, page: Option<Pagination>) -> Result<(Vec<Tenant>, u32)>;
}
