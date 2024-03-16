mod schema;
pub use schema::parse_schema;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid schema provided.")]
    InvalidSchema,
    #[error("Parsing error")]
    ParsingError,
}
