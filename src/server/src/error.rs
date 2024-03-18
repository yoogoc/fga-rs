use aide::{
    openapi::{MediaType, Response as AideResponse},
    OperationOutput,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use indexmap::IndexMap;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(unused)]
pub enum ServerError {
    #[error("Server already started: {server}")]
    AlreadyStarted { server: String },
    #[error("Get lock error")]
    Locked,
    #[error("Database connect error")]
    DatabaseConnect,
    #[error("parser dsl error")]
    ParserError,
}

pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl OperationOutput for AppError {
    type Inner = Self;
    fn operation_response(
        _ctx: &mut aide::gen::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        Some(AideResponse {
            description: "error".into(),
            content: IndexMap::from_iter([("application/json".into(), MediaType::default())]),
            ..Default::default()
        })
    }

    fn inferred_responses(
        _ctx: &mut aide::gen::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        Vec::new()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub type Result<T, E = AppError> = core::result::Result<T, E>;
