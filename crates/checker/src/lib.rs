pub mod cache_checker;
pub mod local_checker;
pub mod remote_checker;

pub mod error;
mod graph;

use std::collections::HashSet;

use anyhow::Result;
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
}

pub trait Checker {
    fn check(&self, req: CheckRequest) -> Result<CheckResult>;
    // call when finish a request
    fn close(&self);
}
