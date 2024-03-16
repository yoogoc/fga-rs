mod schema;
pub use schema::parse_schema;
pub use schema::{Associations, Definition, Permission, PermissionSet, Permissions, Relation, Schema, SubjectSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid schema provided.")]
    InvalidSchema,
    #[error("Parsing error")]
    ParsingError,
}
