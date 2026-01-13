//! High-level asynchronous Jenkins client.

use crate::{
    Auth, BodySnippetConfig, Error, HttpError, RequestHookContext, api,
    transport::{
        TransportBody, TransportRequest,
        async_transport::{DynAsyncTransport, ReqwestAsync},
        middleware::{CrumbAsync, HookAsync, RetryAsync, RetryConfig},
        request::{Request, Response},
    },
    util::{
        diagnostics,
        redact::redact_text,
        url::{endpoint_url, normalize_base_url, sanitize_url_for_error},
    },
};
use http::HeaderMap;
use serde::de::DeserializeOwned;
use std::{sync::Arc, time::Duration};
use url::Url;

#[cfg(feature = "tracing")]
use tracing::field;

#[derive(Clone, Copy, Debug)]
struct CrumbConfig {
    ttl: Duration,
}

const DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Configures and constructs [`Client`].
pub struct ClientBuilder {
    base_url: Url,
    auth: Option<Auth>,
    insecure: bool,
    user_agent: String,
    timeout: Duration,
    connect_timeout: Duration,
    no_proxy: bool,
    retry: Option<RetryConfig>,
    crumb: Option<CrumbConfig>,
    default_headers: HeaderMap,
    body_snippet: BodySnippetConfig,
    request_hook: Option<crate::RequestHook>,
}

impl ClientBuilder {
    /// Create a builder with opinionated defaults.
    fn try_new(base: impl AsRef<str>) -> Result<Self, Error> {
        let base_url = normalize_base_url(base.as_ref())?;
        Ok(Self {
            base_url,
            auth: None,
            insecure: false,
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            no_proxy: false,
            retry: None,
            crumb: None,
            default_headers: HeaderMap::new(),
            body_snippet: BodySnippetConfig::default(),
            request_hook: None,
        })
    }

    /// Apply an authentication strategy.
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Apply HTTP basic authentication credentials.
    pub fn auth_basic(mut self, user: impl Into<String>, token: impl Into<String>) -> Self {
        self.auth = Some(Auth::basic(user, token));
        self
    }

    /// Ignore system proxy environment variables.
    pub fn no_system_proxy(mut self) -> Self {
        self.no_proxy = true;
        self
    }

    /// Accept invalid TLS certificates (**dangerous**).
    pub fn danger_accept_invalid_certs(mut self, yes: bool) -> Self {
        self.insecure = yes;
        self
    }

    /// Override the default `User-Agent` header.
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }

    /// Adjust the per-request timeout.
    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }

    /// Adjust the connection establishment timeout.
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Add a default header applied to every request.
    pub fn default_header(
        mut self,
        name: http::header::HeaderName,
        value: http::HeaderValue,
    ) -> Self {
        self.default_headers.insert(name, value);
        self
    }

    /// Add a set of default headers applied to every request.
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers.extend(headers);
        self
    }

    /// Enable/disable capturing `body_snippet` on errors and decode failures.
    pub fn capture_body_snippet(mut self, enabled: bool) -> Self {
        self.body_snippet.enabled = enabled;
        self
    }

    /// Set max bytes to keep for `body_snippet`.
    pub fn max_body_snippet_bytes(mut self, max_bytes: usize) -> Self {
        self.body_snippet.max_bytes = max_bytes;
        self
    }

    /// Wrap the transport with a conservative retry policy.
    pub fn with_retry(mut self, max_retries: usize, base_delay: Duration) -> Self {
        self.retry = Some(RetryConfig::new(max_retries, base_delay));
        self
    }

    /// Use a custom retry configuration.
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry = Some(config);
        self
    }

    /// Enable CSRF crumb fetching on the first non-GET request.
    pub fn with_crumb(mut self, ttl: Duration) -> Self {
        self.crumb = Some(CrumbConfig { ttl });
        self
    }

    /// Add a hook invoked for every request attempt (including retries).
    pub fn request_hook<F>(mut self, hook: F) -> Self
    where
        F: for<'a> Fn(RequestHookContext<'a>) -> Result<(), Error> + Send + Sync + 'static,
    {
        self.request_hook = Some(Arc::new(hook));
        self
    }

    /// Finalise configuration and build the client.
    pub fn build(self) -> Result<Client, Error> {
        let base = self.base_url;

        let mut transport: DynAsyncTransport = Arc::new(ReqwestAsync::try_new(
            self.insecure,
            &self.user_agent,
            self.timeout,
            self.connect_timeout,
            self.no_proxy,
        )?);

        if let Some(hook) = self.request_hook {
            transport = Arc::new(HookAsync::new(transport, hook));
        }

        if let Some(retry) = self.retry {
            transport = Arc::new(RetryAsync::new(transport, retry));
        }

        if let Some(crumb) = self.crumb {
            transport = Arc::new(CrumbAsync::new(
                transport,
                base.clone(),
                self.auth.clone(),
                self.default_headers.clone(),
                crumb.ttl,
                self.timeout,
                self.body_snippet,
            ));
        }

        Ok(Client {
            inner: Arc::new(Inner {
                base,
                auth: self.auth,
                timeout: self.timeout,
                default_headers: self.default_headers,
                body_snippet: self.body_snippet,
                transport,
            }),
        })
    }
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    base: Url,
    auth: Option<Auth>,
    timeout: Duration,
    default_headers: HeaderMap,
    body_snippet: BodySnippetConfig,
    transport: DynAsyncTransport,
}

impl Client {
    pub fn builder(base: impl AsRef<str>) -> Result<ClientBuilder, Error> {
        ClientBuilder::try_new(base)
    }

    pub fn new(base: impl AsRef<str>) -> Result<Self, Error> {
        Self::builder(base)?.build()
    }

    #[must_use]
    pub fn system(&self) -> api::SystemService {
        api::SystemService::new(self.clone())
    }

    #[must_use]
    pub fn jobs(&self) -> api::JobsService {
        api::JobsService::new(self.clone())
    }

    #[must_use]
    pub fn queue(&self) -> api::QueueService {
        api::QueueService::new(self.clone())
    }

    #[must_use]
    pub fn computers(&self) -> api::ComputersService {
        api::ComputersService::new(self.clone())
    }

    #[must_use]
    pub fn views(&self) -> api::ViewsService {
        api::ViewsService::new(self.clone())
    }

    #[must_use]
    pub fn users(&self) -> api::UsersService {
        api::UsersService::new(self.clone())
    }

    #[must_use]
    pub fn people(&self) -> api::PeopleService {
        api::PeopleService::new(self.clone())
    }

    pub(crate) async fn send_json<T: DeserializeOwned + Send + 'static>(
        &self,
        req: Request,
    ) -> Result<T, Error> {
        let url = endpoint_url(&self.inner.base, req.segments.iter().map(|s| s.as_str()))?;
        let resp = self.execute_request(&req).await?;
        resp.json().map_err(|source| Error::Decode {
            status: resp.status,
            method: req.method,
            path: url.path().to_string().into_boxed_str(),
            request_id: diagnostics::request_id(&resp.headers),
            body_snippet: diagnostics::body_snippet(
                &resp.body,
                self.inner.body_snippet,
                self.inner.auth.as_ref(),
            ),
            source: Box::new(source),
        })
    }

    pub(crate) async fn send_text(&self, req: Request) -> Result<String, Error> {
        let resp = self.execute_request(&req).await?;
        Ok(String::from_utf8_lossy(&resp.body).into_owned())
    }

    pub(crate) async fn send_bytes(&self, req: Request) -> Result<Vec<u8>, Error> {
        let resp = self.execute_request(&req).await?;
        Ok(resp.body)
    }

    pub(crate) async fn send_unit(&self, req: Request) -> Result<(), Error> {
        let _ = self.execute_request(&req).await?;
        Ok(())
    }

    pub(crate) async fn send_response(&self, req: Request) -> Result<Response, Error> {
        self.execute_request(&req).await
    }

    #[cfg(feature = "unstable-raw")]
    pub async fn execute(&self, req: &Request) -> Result<Response, Error> {
        self.execute_request(req).await
    }

    pub(crate) async fn execute_request(&self, req: &Request) -> Result<Response, Error> {
        #[cfg(feature = "metrics")]
        let _inflight = crate::transport::metrics::InFlightGuard::new();

        if req.body.is_some() && !req.form.is_empty() {
            return Err(Error::InvalidConfig {
                message: "request.body and request.form are mutually exclusive".into(),
                source: None,
            });
        }

        let url = endpoint_url(&self.inner.base, req.segments.iter().map(|s| s.as_str()))?;

        let mut headers = self.inner.default_headers.clone();
        if let Some(auth) = &self.inner.auth {
            auth.apply(&mut headers)?;
        }
        headers.extend(req.headers.clone());

        let body = req.body.clone().map(|body| TransportBody {
            bytes: body.bytes,
            content_type: body.content_type,
        });

        #[cfg(any(feature = "tracing", feature = "metrics"))]
        let start = std::time::Instant::now();
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!(
            "jenkins.request",
            http.method = %req.method,
            http.host = %self.inner.base.host_str().unwrap_or_default(),
            http.path = %url.path(),
            http.status = field::Empty,
            request_id = field::Empty,
            retries = field::Empty,
            latency_ms = field::Empty,
            error_kind = field::Empty,
        );
        #[cfg(feature = "tracing")]
        let _enter = span.enter();

        let timeout = req.timeout_override.unwrap_or(self.inner.timeout);
        let resp = match self
            .inner
            .transport
            .send(TransportRequest {
                method: req.method.clone(),
                url: url.clone(),
                headers,
                query: req.query.clone(),
                form: req.form.clone(),
                body,
                timeout,
            })
            .await
        {
            Ok(resp) => resp,
            Err(err) => {
                #[cfg(feature = "metrics")]
                crate::transport::metrics::record_outcome(
                    &req.method,
                    err.status(),
                    start.elapsed(),
                    0,
                    Some(err.kind()),
                );
                #[cfg(feature = "tracing")]
                {
                    span.record("error_kind", field::debug(err.kind()));
                    span.record("latency_ms", start.elapsed().as_millis() as i64);
                }
                return Err(err);
            }
        };

        let request_id = diagnostics::request_id(&resp.headers);

        #[cfg(feature = "tracing")]
        {
            span.record("http.status", resp.status.as_u16() as i64);
            span.record("retries", resp.meta.retries as i64);
            span.record("latency_ms", start.elapsed().as_millis() as i64);
            if let Some(rid) = request_id.as_deref() {
                span.record("request_id", field::display(rid));
            }
        }

        if resp.status.is_client_error() || resp.status.is_server_error() {
            let safe_url = sanitize_url_for_error(&url);
            let message = diagnostics::extract_message(&resp.body)
                .map(|msg| redact_text(msg.into(), self.inner.auth.as_ref()).into_boxed_str());
            let http_error = HttpError {
                status: resp.status,
                method: req.method.clone(),
                url: Box::new(safe_url),
                message,
                request_id,
                body_snippet: diagnostics::body_snippet(
                    &resp.body,
                    self.inner.body_snippet,
                    self.inner.auth.as_ref(),
                ),
            };

            let retry_after = crate::transport::middleware::retry::parse_retry_after(
                &resp.headers,
                std::time::SystemTime::now(),
            );
            let err = Error::from_http(http_error, retry_after);

            #[cfg(feature = "metrics")]
            crate::transport::metrics::record_outcome(
                &req.method,
                err.status(),
                start.elapsed(),
                resp.meta.retries,
                Some(err.kind()),
            );
            #[cfg(feature = "tracing")]
            span.record("error_kind", field::debug(err.kind()));

            return Err(err);
        }

        let _retries = resp.meta.retries;
        let response = Response {
            status: resp.status,
            headers: resp.headers,
            body: resp.body,
            #[cfg(feature = "unstable-raw")]
            retries: _retries,
        };

        #[cfg(feature = "metrics")]
        crate::transport::metrics::record_outcome(
            &req.method,
            Some(response.status),
            start.elapsed(),
            _retries,
            None,
        );

        Ok(response)
    }
}
