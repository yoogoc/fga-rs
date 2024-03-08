use axum::{
    body::Body,
    extract::{Path, Query, State},
    Json,
};
use checker::{CheckRequest, CheckResult, CheckerRef};
use protocol::{Tuple, TupleKey};
use serde::{Deserialize, Serialize};
use storage::{Pagination, RelationshipTupleReaderRef, RelationshipTupleWriterRef, TupleFilter};

use crate::error::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReadResult {
    tuples: Vec<Tuple>,
    total: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CheckReq {
    tuple_key: TupleKey,
    contextual_tuples: Vec<TupleKey>,
}

impl From<(Vec<Tuple>, Option<u64>)> for ReadResult {
    fn from(t: (Vec<Tuple>, Option<u64>)) -> Self {
        Self {
            tuples: t.0,
            total: t.1.map(|x| x as u32),
        }
    }
}

#[axum::debug_handler]
pub async fn read(
    State(state): State<RelationshipTupleReaderRef>,
    Path(tenant_id): Path<String>,
    Query(filter): Query<Option<TupleFilter>>,
    Query(page): Query<Option<Pagination>>,
) -> Result<Json<ReadResult>> {
    let filter = filter.unwrap_or_default();
    let result = state.list(&tenant_id, filter, page).await?;
    Ok(Json(result.into()))
}

#[axum::debug_handler]
pub async fn write_save(
    State(state): State<RelationshipTupleWriterRef>,
    Path(tenant_id): Path<String>,
    Json(tuples): Json<Vec<Tuple>>,
) -> Result<Json<()>> {
    state.save(&tenant_id, tuples).await?;
    Ok(Json(()))
}

#[axum::debug_handler]
pub async fn write_delete(
    State(state): State<RelationshipTupleWriterRef>,
    Path(tenant_id): Path<String>,
    Json(filter): Json<TupleFilter>,
) -> Result<Json<()>> {
    let _ = state.delete(&tenant_id, filter).await?;
    Ok(Json(()))
}

#[axum::debug_handler]
pub async fn watch(Path(tenant_id): Path<String>) -> Result<Json<ReadResult>> {
    unimplemented!()
}

// define check will fail
#[axum::debug_handler]
pub async fn check_x(
    State(state): State<CheckerRef>,
    Path(tenant_id): Path<String>,
    Json(req): Json<CheckReq>,
) -> Result<Json<CheckResult>> {
    let cr = CheckRequest {
        // TODO typesystem convert
        tuple_key: req.tuple_key,
        contextual_tuples: req.contextual_tuples,
        ..Default::default()
    };
    let result = state.check(cr).await?;
    Ok(Json(result))
}

#[axum::debug_handler]
pub async fn expand(
    State(state): State<RelationshipTupleReaderRef>,
    Path(tenant_id): Path<String>,
) -> Result<Json<ReadResult>> {
    todo!()
}
