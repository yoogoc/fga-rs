use anyhow::Result;

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
    fn read(tenant_id: String, filter: TupleFilter, page: Option<Pagination>) -> Result;
}
