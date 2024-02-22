use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Not found relations by object type: {0}")]
    NotFoundRelations(String),
    #[error("Not found relation by relation: {0}")]
    NotFoundRelation(String),
}
