//! Blocking CSRF-Crumb middleware.

use super::retry::parse_retry_after;
use crate::{
    Auth, BodySnippetConfig, Error, HttpError,
    transport::{TransportRequest, blocking_transport::BlockingTransport},
    util::{
        diagnostics,
        redact::redact_text,
        url::{endpoint_url, sanitize_url_for_error},
    },
};
use http::{HeaderMap, HeaderValue, Method};
use serde::Deserialize;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use url::Url;

#[derive(Deserialize)]
struct CrumbResp {
    #[serde(rename = "crumbRequestField")]
    field: String,
    crumb: String,
}

struct CachedCrumb {
    field: String,
    crumb: String,
    ts: Instant,
}

/// Blocking wrapper that injects a crumb header.
#[derive(Clone)]
pub struct CrumbBlocking<T> {
    inner: T,
    base_url: Url,
    auth: Option<Auth>,
    default_headers: HeaderMap,
    ttl: Duration,
    fetch_timeout: Duration,
    body_snippet: BodySnippetConfig,
    cache: Arc<Mutex<Option<CachedCrumb>>>,
}

impl<T: BlockingTransport> CrumbBlocking<T> {
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
            cache: Arc::new(Mutex::new(None)),
        }
    }

    fn fetch_crumb(&self) -> Result<CachedCrumb, Error> {
        let url = endpoint_url(&self.base_url, ["crumbIssuer", "api", "json"])?;
        let url_for_error = url.clone();

        let mut hdrs = self.default_headers.clone();
        if let Some(auth) = &self.auth {
            auth.apply(&mut hdrs)?;
        }

        let resp = self.inner.send(TransportRequest {
            method: Method::GET,
            url,
            headers: hdrs,
            query: vec![],
            form: vec![],
            body: None,
            timeout: self.fetch_timeout,
        })?;
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

impl<T: BlockingTransport> BlockingTransport for CrumbBlocking<T> {
    fn send(
        &self,
        mut req: TransportRequest,
    ) -> Result<crate::transport::TransportResponse, Error> {
        if req.method != Method::GET {
            let mut guard = match self.cache.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let expired = guard
                .as_ref()
                .map(|c| c.ts.elapsed() > self.ttl)
                .unwrap_or(true);

            if expired {
                *guard = Some(self.fetch_crumb()?);
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

        self.inner.send(req)
    }
}
