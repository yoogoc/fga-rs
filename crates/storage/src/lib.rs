use anyhow::Result;
use protocol::{AuthzModel, Tenant, Tuple};

pub struct Pagination {
    pub size: u32,
    pub page: u32,
}

pub struct TupleFilter {
    pub object_type_eq: String,
    pub object_id_eq: String,
    pub object_id_in: Vec<String>,
    pub relation_eq: String,
    pub user_type_eq: String,
    pub user_id_eq: String,
    pub user_id_in: Vec<String>,
    pub user_relation_eq: String,
}

pub trait RelationshipTupleReader {
    fn read(
        &self,
        tenant_id: String,
        filter: TupleFilter,
        page: Option<Pagination>,
    ) -> Result<(Vec<Tuple>, u32)>;
}

pub trait RelationshipTupleWriter {
    fn save(&self, tuples: Vec<Tuple>) -> Result<()>;
    fn delete(&self, filter: TupleFilter) -> Result<()>;
}

pub trait AuthzModelReader {
    fn read_latest(&self, tenant_id: String) -> Result<AuthzModel>;
    fn read(&self, tenant_id: String, page: Option<Pagination>) -> Result<(Vec<AuthzModel>, u32)>;
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
