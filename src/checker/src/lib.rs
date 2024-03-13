pub mod cache_checker;
pub mod local_checker;
pub mod remote_checker;
mod stream;
use async_trait::async_trait;
use futures::Future;
use serde::{Deserialize, Serialize};

pub mod error;
mod graph;

use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use graph::ResolutionMetadata;
use protocol::{TupleKey, Typesystem};

pub use cache_checker::CacheChecker;
pub use local_checker::LocalChecker;
pub use remote_checker::RemoteChecker;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CheckRequest {
    pub typesystem: Typesystem,
    pub tuple_key: TupleKey,
    pub contextual_tuples: Vec<TupleKey>,
    pub resolution_metadata: ResolutionMetadata,
    pub visited_paths: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub struct CheckResult {
    pub allow: bool,
    pub resolution_metadata: ResolutionMetadata,
}
impl CheckResult {
    fn new(allow: bool) -> CheckResult {
        Self {
            allow,
            resolution_metadata: ResolutionMetadata::default(),
        }
    }
    fn new_dqc(allow: bool, count: u32) -> CheckResult {
        Self {
            allow,
            resolution_metadata: ResolutionMetadata {
                datastore_query_count: count,
                ..Default::default()
            },
        }
    }
}

#[async_trait]
pub trait Checker: Send + Sync {
    async fn check(&self, req: CheckRequest) -> Result<CheckResult>;
    // call when finish a request
    async fn close(&self);
}

pub type CheckerRef = Arc<dyn Checker>;

async fn union_check<F>(count: usize, f: impl Fn(usize) -> F) -> Result<CheckResult>
where
    F: Future<Output = Result<CheckResult>>,
{
    let mut db_read = 0u32;
    for i in 0..count {
        let result = f(i).await;
        if let Ok(cr) = result {
            db_read += cr.resolution_metadata.datastore_query_count;
            if cr.allow {
                return result;
            }
        }
    }
    Ok(CheckResult::new_dqc(false, db_read))
}

async fn intersection_check<F>(count: usize, f: impl Fn(usize) -> F) -> Result<CheckResult>
where
    F: Future<Output = Result<CheckResult>>,
{
    let mut db_read = 0u32;
    for i in 0..count {
        let result = f(i).await;
        if let Ok(res) = result {
            db_read += res.resolution_metadata.datastore_query_count;
            if !res.allow {
                return Ok(CheckResult::new_dqc(false, db_read));
            }
        }
    }

    Ok(CheckResult::new_dqc(true, db_read))
}

async fn exclusion_check<F>(base: F, subtract: F) -> Result<CheckResult>
where
    F: Future<Output = Result<CheckResult>>,
{
    let mut db_read = 0u32;
    let base_result = base.await?;
    db_read += base_result.resolution_metadata.datastore_query_count;
    if !base_result.allow {
        return Ok(CheckResult::new_dqc(false, db_read));
    }

    let subtract_result = subtract.await?;
    db_read += subtract_result.resolution_metadata.datastore_query_count;
    if subtract_result.allow {
        return Ok(CheckResult::new_dqc(false, db_read));
    }
    Ok(CheckResult::new_dqc(true, db_read))
}
