//! Async CSRF-Crumb middleware.
//
//! * Lazily fetches `/crumbIssuer/api/json` on the **first** non-GET request.
//! * Caches the crumb header for `ttl`; subsequent POST/PUT reuse it.
//! * Thread-safe via `Arc<RwLock<ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦>>`.

use crate::{core::error::JenkinsError, transport::async_impl::AsyncTransport};
use async_trait::async_trait;
use base64::Engine; // bring `encode()` into scope
use http::{Method, StatusCode};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use url::Url;

/// JSON payload of `/crumbIssuer/api/json`
#[derive(Deserialize)]
struct CrumbResp {
    #[serde(rename = "crumbRequestField")]
    field: String,
    crumb: String,
}

/// In-memory cache entry.
struct CachedCrumb {
    field: String,
    crumb: String,
    ts: Instant,
}

/// Async wrapper that injects a valid crumb header.
#[derive(Clone)]
pub struct CrumbAsync<T> {
    inner: T,
    base_url: Url,
    auth_basic: Option<(String, String)>,
    ttl: Duration,
    fetch_timeout: Duration,
    cache: Arc<RwLock<Option<CachedCrumb>>>,
}

impl<T: AsyncTransport> CrumbAsync<T> {
    /// Build a new wrapper.
    pub fn new(
        inner: T,
        base_url: Url,
        auth_basic: Option<(String, String)>,
        ttl: Duration,
        fetch_timeout: Duration,
    ) -> Self {
        Self {
            inner,
            base_url,
            auth_basic,
            ttl,
            fetch_timeout,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// GET `/crumbIssuer/api/json` and return a fresh cache entry.
    async fn fetch_crumb(&self) -> Result<CachedCrumb, JenkinsError> {
        let mut url = self.base_url.clone();
        {
            let mut url_path = url
                .path_segments_mut()
                .expect("Base URL should not be a cannot-be-a-base URL");

            url_path.push("crumbIssuer");
            url_path.push("api");
            url_path.push("json");
        }
        let url = url;

        let url_for_error = url.clone(); // keep a copy for error context

        let mut hdrs = HashMap::new();
        if let Some((u, p)) = &self.auth_basic {
            hdrs.insert(
                "Authorization".into(),
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD.encode(format!("{u}:{p}"))
                ),
            );
        }

        let (code, body) = self
            .inner
            .send(Method::GET, url, hdrs, vec![], vec![], self.fetch_timeout)
            .await?;

        if !code.is_success() {
            return Err(JenkinsError::Http {
                code,
                method: Method::GET,
                url: url_for_error,
                body,
            });
        }

        let json: CrumbResp = serde_json::from_str(&body)?;
        Ok(CachedCrumb {
            field: json.field,
            crumb: json.crumb,
            ts: Instant::now(),
        })
    }
}

#[async_trait]
impl<T: AsyncTransport> AsyncTransport for CrumbAsync<T> {
    async fn send(
        &self,
        method: Method,
        url: Url,
        mut headers: HashMap<String, String>,
        query: Vec<(String, String)>,
        form: Vec<(String, String)>,
        timeout: Duration,
    ) -> Result<(StatusCode, String), JenkinsError> {
        // Non-GET calls need a crumb.
        if method != Method::GET {
            let mut guard = self.cache.write().await;
            let expired = guard
                .as_ref()
                .map(|c| c.ts.elapsed() > self.ttl)
                .unwrap_or(true);

            if expired {
                *guard = Some(self.fetch_crumb().await?);
            }
            if let Some(c) = guard.as_ref() {
                headers.insert(c.field.clone(), c.crumb.clone());
            }
        }

        self.inner
            .send(method, url, headers, query, form, timeout)
            .await
    }
}
