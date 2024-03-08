use crate::error::Result;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use protocol::Tenant;
use serde::{Deserialize, Serialize};
use storage::{Pagination, TenantOperatorRef};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReadResult {
    tenants: Vec<Tenant>,
    total: Option<u32>,
}

impl From<(Vec<Tenant>, Option<u64>)> for ReadResult {
    fn from(t: (Vec<Tenant>, Option<u64>)) -> Self {
        Self {
            tenants: t.0,
            total: t.1.map(|x| x as u32),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateRequest {
    id: String,
    name: String,
}

#[axum::debug_handler]
pub async fn list(
    State(state): State<TenantOperatorRef>,
    Query(page): Query<Option<Pagination>>,
) -> Result<Json<ReadResult>> {
    let result = state.list(page).await?;
    Ok(Json(result.into()))
}

#[axum::debug_handler]
pub async fn create(State(state): State<TenantOperatorRef>, Json(cr): Json<CreateRequest>) -> Result<Json<()>> {
    let _ = state.create(cr.id, cr.name).await?;
    Ok(Json(()))
}

#[axum::debug_handler]
pub async fn get(State(state): State<TenantOperatorRef>, Path(id): Path<String>) -> Result<Json<Tenant>> {
    let result = state.get(id).await?;
    Ok(Json(result))
}

#[axum::debug_handler]
pub async fn delete(State(state): State<TenantOperatorRef>, Path(id): Path<String>) -> Result<Json<()>> {
    let _ = state.delete(id).await?;
    Ok(Json(()))
}
