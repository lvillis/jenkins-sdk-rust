use crate::Error;
use http::{HeaderMap, HeaderValue, Method};
use std::sync::Arc;
use url::Url;

/// Request hook context passed to `ClientBuilder::request_hook`.
///
/// The hook can inspect request parts and mutate headers before the request is sent.
pub struct RequestHookContext<'a> {
    pub method: &'a Method,
    /// URL without query/fragment.
    pub url: &'a Url,
    pub headers: &'a mut HeaderMap,
    /// Query pairs appended by the transport.
    pub query: &'a [(String, String)],
    /// Form pairs encoded by the transport when body is absent.
    pub form: &'a [(String, String)],
    pub body: Option<&'a [u8]>,
    pub content_type: Option<&'a HeaderValue>,
}

pub type RequestHook =
    Arc<dyn for<'a> Fn(RequestHookContext<'a>) -> Result<(), Error> + Send + Sync + 'static>;
