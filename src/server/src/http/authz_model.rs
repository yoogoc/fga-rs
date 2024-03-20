use crate::error::{Result, ServerError};
use axum::extract::{Json, Path, Query, State};
use protocol::Tenant;
use schema::Schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use storage::{AuthzModelReaderRef, AuthzModelWriterRef, Pagination};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Model {
    id: String,
    model: Schema,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReadResult {
    models: Vec<Model>,
    total: Option<u32>,
}

impl From<(Vec<(String, Schema)>, Option<u64>)> for ReadResult {
    fn from((schemas, total): (Vec<(String, Schema)>, Option<u64>)) -> Self {
        Self {
            models: schemas
                .iter()
                .map(|(id, schema)| {
                    return Model {
                        id: id.to_string(),
                        model: schema.to_owned(),
                    };
                })
                .collect(),
            total: total.map(|x| x as u32),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct CreateByDslRequest {
    dsl: String,
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
    Json(cr): Json<Schema>,
) -> Result<Json<()>> {
    let _ = state.save(tenant_id, cr).await?;
    Ok(Json(()))
}

#[axum::debug_handler]
pub async fn create_by_dsl(
    State(state): State<AuthzModelWriterRef>,
    Path(tenant_id): Path<String>,
    Json(cr): Json<CreateByDslRequest>,
) -> Result<Json<()>> {
    let cr = schema::parse(&cr.dsl).map_err(|e| {
        error!("{:?}", e);
        return ServerError::ParserError;
    })?;
    let _ = state.save(tenant_id, cr.0).await?;
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
