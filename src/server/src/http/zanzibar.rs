use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    Json,
};
use checker::{
    expander::{ExpandTree, Expander, ObjectsExpander, UsersExpander},
    CheckRequest, CheckResult, CheckerRef,
};
use protocol::{Tuple, TupleKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use storage::{AuthzModelReaderRef, Pagination, RelationshipTupleReaderRef, RelationshipTupleWriterRef, TupleFilter};
use tracing::Instrument;

use crate::error::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ReadResult {
    tuples: Vec<Tuple>,
    total: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct CheckReq {
    model_id: Option<String>,
    tuple_key: TupleKey,
    #[serde(default)]
    contextual_tuples: Vec<TupleKey>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ExpandReq {
    model_id: Option<String>,
    relation: String,
    object_type: String,
    object_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ExpandObjectsReq {
    model_id: Option<String>,
    relation: String,
    object_type: String,
    user_type: String,
    user_id: String,
    user_relation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ExpandObjectsResp {
    object_ids: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ExpandUsersReq {
    model_id: Option<String>,
    relation: String,
    object_type: String,
    object_id: String,
    user_type: String,
    user_relation: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct ExpandUsersResp {
    user_ids: HashSet<String>,
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
    filter: Option<Query<TupleFilter>>,
    page: Option<Query<Pagination>>,
) -> Result<Json<ReadResult>> {
    let filter = filter.map(|x| x.0).unwrap_or_default();
    let result = state.list(&tenant_id, filter, page.map(|x| x.0)).await?;
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
#[allow(unused)]
pub async fn watch(Path(tenant_id): Path<String>) -> Result<Json<ReadResult>> {
    unimplemented!()
}

// define check will fail
#[axum::debug_handler]
pub async fn check_x(
    State((checker, model_reader)): State<(CheckerRef, AuthzModelReaderRef)>,
    Path(tenant_id): Path<String>,
    Json(req): Json<CheckReq>,
) -> Result<Json<CheckResult>> {
    let (id, model) = if let Some(model_id) = req.model_id {
        model_reader.get(String::from(&tenant_id), model_id).await?
    } else {
        model_reader.get_latest(String::from(&tenant_id)).await?
    };
    let span = trace_span!("check");

    let cr = CheckRequest {
        tenant_id,
        model_id: id,
        tuple_key: req.tuple_key,
        contextual_tuples: req.contextual_tuples,
        typesystem: model.to_typesystem(),
        ..Default::default()
    };
    let result = checker.check(cr).instrument(span).await?;
    Ok(Json(result))
}

#[axum::debug_handler]
pub async fn expand(
    State((expander, model_reader)): State<(Arc<Expander>, AuthzModelReaderRef)>,
    Path(tenant_id): Path<String>,
    Json(req): Json<ExpandReq>,
) -> Result<Json<ExpandTree>> {
    let (_id, model) = if let Some(model_id) = req.model_id {
        model_reader.get(String::from(&tenant_id), model_id).await?
    } else {
        model_reader.get_latest(String::from(&tenant_id)).await?
    };
    let result = expander
        .expand(
            model.to_typesystem(),
            tenant_id,
            req.relation,
            req.object_type,
            req.object_id,
        )
        .await?;
    Ok(Json(result))
}

#[axum::debug_handler]
pub async fn expand_objects(
    State((expander, model_reader)): State<(Arc<ObjectsExpander>, AuthzModelReaderRef)>,
    Path(tenant_id): Path<String>,
    Json(req): Json<ExpandObjectsReq>,
) -> Result<Json<ExpandObjectsResp>> {
    let (_id, model) = if let Some(model_id) = req.model_id {
        model_reader.get(String::from(&tenant_id), model_id).await?
    } else {
        model_reader.get_latest(String::from(&tenant_id)).await?
    };

    let object_ids = expander
        .objects(
            model.to_typesystem(),
            tenant_id,
            req.relation,
            req.object_type,
            req.user_type,
            req.user_id,
            req.user_relation,
        )
        .await?;

    Ok(Json(ExpandObjectsResp { object_ids }))
}

#[axum::debug_handler]
pub async fn expand_users(
    State((expander, model_reader)): State<(Arc<UsersExpander>, AuthzModelReaderRef)>,
    Path(tenant_id): Path<String>,
    Json(req): Json<ExpandUsersReq>,
) -> Result<Json<ExpandUsersResp>> {
    let (_id, model) = if let Some(model_id) = req.model_id {
        model_reader.get(String::from(&tenant_id), model_id).await?
    } else {
        model_reader.get_latest(String::from(&tenant_id)).await?
    };
    let user_ids = expander
        .users(
            model.to_typesystem(),
            tenant_id,
            req.relation,
            req.object_type,
            req.object_id,
            req.user_type,
            req.user_relation,
        )
        .await?;

    Ok(Json(ExpandUsersResp { user_ids }))
}
