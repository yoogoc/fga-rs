use crate::error::Result;
use axum::extract::{Json, Path, Query, State};
use protocol::{AuthzModel, Tenant};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use storage::{AuthzModelReaderRef, AuthzModelWriterRef, Pagination};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReadResult {
    models: Vec<AuthzModel>,
    total: Option<u32>,
}

impl From<(Vec<AuthzModel>, Option<u64>)> for ReadResult {
    fn from(t: (Vec<AuthzModel>, Option<u64>)) -> Self {
        Self {
            models: t.0,
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
    State(state): State<AuthzModelReaderRef>,
    Path(tenant_id): Path<String>,
    page: Option<Query<Pagination>>,
) -> Result<Json<ReadResult>> {
    let result = state.list(tenant_id, page.map(|p| p.0)).await?;
    Ok(Json(result.into()))
}

#[axum::debug_handler]
pub async fn create(
    State(state): State<AuthzModelWriterRef>,
    Path(tenant_id): Path<String>,
    Json(cr): Json<AuthzModel>,
) -> Result<Json<()>> {
    let _ = state.save(tenant_id, cr).await?;
    Ok(Json(()))
}

#[axum::debug_handler]
#[allow(unused)]
pub async fn get(
    State(state): State<AuthzModelReaderRef>,
    Path((tenant_id, id)): Path<(String, String)>,
) -> Result<Json<Tenant>> {
    // let result = state.get(id).await?;
    // Ok(Json(result))
    todo!()
}
