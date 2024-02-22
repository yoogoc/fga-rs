pub mod cache_checker;
pub mod local_checker;
pub mod remote_checker;

pub mod error;
mod graph;

use std::collections::HashSet;

use anyhow::{Ok, Result};
use graph::ResolutionMetadata;
use protocol::{TupleKey, Typesystem};

#[derive(Debug, Clone, PartialEq)]
pub struct CheckRequest {
    pub typesystem: Typesystem,
    pub tuple_key: TupleKey,
    pub contextual_tuples: Vec<TupleKey>,
    pub resolution_metadata: ResolutionMetadata,
    pub visited_paths: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
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

pub trait Checker {
    fn check(&self, req: CheckRequest) -> Result<CheckResult>;
    // call when finish a request
    fn close(&self);
}

// TODO async
pub fn union_check(handlers: Vec<impl Fn() -> Result<CheckResult>>) -> Result<CheckResult> {
    let mut db_read = 0u32;
    for handler in handlers {
        let res = handler()?;
        db_read += res.resolution_metadata.datastore_query_count;
        if res.allow {
            return Ok(CheckResult::new_dqc(true, db_read));
        }
    }

    Ok(CheckResult::new_dqc(false, db_read))
}

pub fn intersection_check(handlers: Vec<impl Fn() -> Result<CheckResult>>) -> Result<CheckResult> {
    let mut db_read = 0u32;
    for handler in handlers {
        let res = handler()?;
        db_read += res.resolution_metadata.datastore_query_count;
        if !res.allow {
            return Ok(CheckResult::new_dqc(false, db_read));
        }
    }

    Ok(CheckResult::new_dqc(true, db_read))
}

pub fn exclusion_check(
    base: impl Fn() -> Result<CheckResult>,
    subtract: impl Fn() -> Result<CheckResult>,
) -> Result<CheckResult> {
    let mut db_read = 0u32;
    let base_result = base()?;
    db_read += base_result.resolution_metadata.datastore_query_count;
    if !base_result.allow {
        return Ok(CheckResult::new_dqc(false, db_read));
    }

    let subtract_result = subtract()?;
    db_read += subtract_result.resolution_metadata.datastore_query_count;
    if subtract_result.allow {
        return Ok(CheckResult::new_dqc(false, db_read));
    }
    Ok(CheckResult::new_dqc(true, db_read))
}
