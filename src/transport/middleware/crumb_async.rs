//! Async CSRF-Crumb middleware.
//
//! * Lazily fetches `/crumbIssuer/api/json` on the **first** non-GET request.
//! * Caches the crumb header for `ttl`; subsequent POST/PUT reuse it.
//! * Thread-safe via `Arc<RwLock<Option<_>>>`.

use super::retry::parse_retry_after;
use crate::{
    Auth, BodySnippetConfig, Error, HttpError,
    transport::{TransportRequest, async_transport::AsyncTransport},
    util::{
        diagnostics,
        redact::redact_text,
        url::{endpoint_url, sanitize_url_for_error},
    },
};
use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, Method};
use serde::Deserialize;
use std::{
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
    auth: Option<Auth>,
    default_headers: HeaderMap,
    ttl: Duration,
    fetch_timeout: Duration,
    body_snippet: BodySnippetConfig,
    cache: Arc<RwLock<Option<CachedCrumb>>>,
}

impl<T: AsyncTransport> CrumbAsync<T> {
    /// Build a new wrapper.
    pub fn new(
        inner: T,
        base_url: Url,
        auth: Option<Auth>,
        default_headers: HeaderMap,
        ttl: Duration,
        fetch_timeout: Duration,
        body_snippet: BodySnippetConfig,
    ) -> Self {
        Self {
            inner,
            base_url,
            auth,
            default_headers,
            ttl,
            fetch_timeout,
            body_snippet,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// GET `/crumbIssuer/api/json` and return a fresh cache entry.
    async fn fetch_crumb(&self) -> Result<CachedCrumb, Error> {
        let url = endpoint_url(&self.base_url, ["crumbIssuer", "api", "json"])?;
        let url_for_error = url.clone(); // keep a copy for error context

        let mut hdrs = self.default_headers.clone();
        if let Some(auth) = &self.auth {
            auth.apply(&mut hdrs)?;
        }

        let resp = self
            .inner
            .send(TransportRequest {
                method: Method::GET,
                url,
                headers: hdrs,
                query: vec![],
                form: vec![],
                body: None,
                timeout: self.fetch_timeout,
            })
            .await?;
        let (code, body, headers) = (resp.status, resp.body, resp.headers);
        let request_id = diagnostics::request_id(&headers);

        if !code.is_success() {
            let safe_url = sanitize_url_for_error(&url_for_error);
            let message = diagnostics::extract_message(&body)
                .map(|msg| redact_text(msg.into(), self.auth.as_ref()).into_boxed_str());

            let http_error = HttpError {
                status: code,
                method: Method::GET,
                url: Box::new(safe_url),
                message,
                request_id,
                body_snippet: diagnostics::body_snippet(
                    &body,
                    self.body_snippet,
                    self.auth.as_ref(),
                ),
            };
            let retry_after = parse_retry_after(&headers, std::time::SystemTime::now());
            return Err(Error::from_http(http_error, retry_after));
        }

        let json: CrumbResp = match serde_json::from_slice(&body) {
            Ok(json) => json,
            Err(err) => {
                return Err(Error::Decode {
                    status: code,
                    method: Method::GET,
                    path: url_for_error.path().to_string().into_boxed_str(),
                    request_id,
                    body_snippet: diagnostics::body_snippet(
                        &body,
                        self.body_snippet,
                        self.auth.as_ref(),
                    ),
                    source: Box::new(err),
                });
            }
        };
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
        mut req: TransportRequest,
    ) -> Result<crate::transport::TransportResponse, Error> {
        // Non-GET calls need a crumb.
        if req.method != Method::GET {
            let mut guard = self.cache.write().await;
            let expired = guard
                .as_ref()
                .map(|c| c.ts.elapsed() > self.ttl)
                .unwrap_or(true);

            if expired {
                *guard = Some(self.fetch_crumb().await?);
            }
            if let Some(c) = guard.as_ref() {
                let value =
                    HeaderValue::from_str(&c.crumb).map_err(|err| Error::InvalidConfig {
                        message: "invalid crumb header value".into(),
                        source: Some(Box::new(err)),
                    })?;
                let name =
                    http::header::HeaderName::from_bytes(c.field.as_bytes()).map_err(|err| {
                        Error::InvalidConfig {
                            message: "invalid crumb header name".into(),
                            source: Some(Box::new(err)),
                        }
                    })?;
                req.headers.insert(name, value);
            }
        }

        self.inner.send(req).await
    }
}
