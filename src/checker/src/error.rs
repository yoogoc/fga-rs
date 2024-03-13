use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckerError {
    #[error("Not found _this type by object type: {object_type}, relation: {relation}")]
    NotFoundThisTypes { object_type: String, relation: String },
}
