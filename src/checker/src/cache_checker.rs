use anyhow::Ok;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use moka::sync::Cache;
use tracing::Instrument;

use crate::{CheckRequest, CheckResult, Checker, CheckerRef};

pub struct CacheChecker {
    delegate: CheckerRef,
    cache: Cache<String, bool>,
}

#[async_trait]
impl Checker for CacheChecker {
    async fn check(&self, req: CheckRequest) -> anyhow::Result<CheckResult> {
        let span = trace_span!("cache-checker");
        let _enter = span.enter();
        let key = self.request_cache_key(&req);
        if let Some(allow) = self.cache.get(&key) {
            trace!("Hit Cache, key is: {}, result is : {}", &key, allow);
            Ok(CheckResult::new(allow))
        } else {
            let checker = self.delegate.clone();
            trace!("Miss Cache, enter {} checker", checker.name());
            let resp = checker.check(req).instrument(span.clone()).await?;
            self.cache.insert(key, resp.allow);
            Ok(CheckResult::new(resp.allow))
        }
    }

    async fn close(&self) {
        self.cache.invalidate_all();
    }

    fn name(&self) -> &str {
        "cache"
    }
}

impl CacheChecker {
    pub fn new(delegate: CheckerRef) -> Self {
        let cache = Cache::new(100);
        Self { delegate, cache }
    }

    fn request_cache_key(&self, req: &CheckRequest) -> String {
        let mut contextual_tuples_cache_key = String::new();
        for tk in req.contextual_tuples.clone() {
            let key = format!("/{}", tk.cache_key());
            contextual_tuples_cache_key.push_str(key.as_str());
        }
        STANDARD.encode(format!(
            "{}/{}/{}{}",
            req.tenant_id,
            req.model_id,
            req.tuple_key.cache_key(),
            contextual_tuples_cache_key
        ))
    }
}
