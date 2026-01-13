//! HTTP transport layers.
//!
//! * Async transport uses `reqwest`.
//! * Blocking transport uses `ureq` so `--features blocking` does not pull an async runtime.
//! * Both enable cookie-store so that `JSESSIONID` persists for CSRF crumbs.

use http::{HeaderMap, HeaderValue, Method, StatusCode};
use std::time::Duration;
use url::Url;

#[cfg(feature = "metrics")]
pub(crate) mod metrics;
pub(crate) mod middleware;
pub(crate) mod request;

#[cfg(feature = "async")]
pub mod async_transport;
#[cfg(feature = "blocking")]
pub mod blocking_transport;

#[derive(Clone, Debug, Default)]
pub struct ResponseMeta {
    pub retries: usize,
}

#[derive(Clone, Debug)]
pub struct TransportResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub meta: ResponseMeta,
}

#[derive(Clone, Debug)]
pub struct TransportBody {
    pub bytes: Vec<u8>,
    pub content_type: Option<HeaderValue>,
}

#[derive(Clone, Debug)]
pub struct TransportRequest {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
    pub query: Vec<(String, String)>,
    pub form: Vec<(String, String)>,
    pub body: Option<TransportBody>,
    pub timeout: Duration,
}
