use anyhow::Ok;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use moka::sync::Cache;

use crate::{CheckRequest, CheckResult, Checker};

pub struct CacheChecker {
    delegate: Box<dyn Checker + Send + Sync>,
    cache: Cache<String, bool>,
}

#[async_trait]
impl Checker for CacheChecker {
    async fn check(&self, req: CheckRequest) -> anyhow::Result<CheckResult> {
        let key = self.request_cache_key(&req);
        if let Some(allow) = self.cache.get(&key) {
            Ok(CheckResult::new(allow))
        } else {
            let resp = self.delegate.check(req).await?;
            self.cache.insert(key, resp.allow);
            Ok(CheckResult::new(resp.allow))
        }
    }

    async fn close(&self) {
        self.cache.invalidate_all();
    }
}

impl CacheChecker {
    fn request_cache_key(&self, req: &CheckRequest) -> String {
        let mut contextual_tuples_cache_key = String::new();
        for tk in req.contextual_tuples.clone() {
            let key = format!("/{}", tk.cache_key());
            contextual_tuples_cache_key.push_str(key.as_str());
        }
        STANDARD.encode(format!(
            "{}/{}/{}{}",
            req.typesystem.tenant_id,
            req.typesystem.model_id,
            req.tuple_key.cache_key(),
            contextual_tuples_cache_key
        ))
    }
}
