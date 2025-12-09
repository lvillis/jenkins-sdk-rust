//! High-level asynchronous Jenkins client.

use crate::{
    core::{Endpoint, JenkinsError},
    middleware::{CrumbAsync, RetryAsync},
    transport::async_impl::{AsyncTransport, DefaultAsyncTransport},
};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use http::Method;
use std::{borrow::Cow, collections::HashMap, time::Duration};
use url::Url;

fn normalize_base_url(raw: &str) -> Result<Url, JenkinsError> {
    let mut url = Url::parse(raw)?;
    let path = url.path().to_string();
    if path != "/" && !path.ends_with('/') {
        url.set_path(&(path + "/"));
    }
    Ok(url)
}

/// Configures and constructs [`JenkinsAsync`].
pub struct JenkinsAsyncBuilder<T = DefaultAsyncTransport> {
    base_url: String,
    auth: Option<(String, String)>,
    insecure: bool,
    timeout: Duration,
    no_proxy: bool,
    transport: T,
}

impl JenkinsAsyncBuilder<DefaultAsyncTransport> {
    /// Create a builder with opinionated defaults.
    fn default_builder(base: impl Into<String>) -> Self {
        Self {
            base_url: base.into(),
            auth: None,
            insecure: false,
            timeout: Duration::from_secs(30),
            no_proxy: false,
            transport: DefaultAsyncTransport::new(
                false,
                "jenkins-sdk-rust",
                Duration::from_secs(30),
                false,
            ),
        }
    }

    /// Rebuild the transport when default flags change.
    fn refresh_transport(&mut self) {
        self.transport = DefaultAsyncTransport::new(
            self.insecure,
            "jenkins-sdk-rust",
            self.timeout,
            self.no_proxy,
        );
    }

    /// Ignore system proxy environment variables.
    pub fn no_system_proxy(mut self) -> Self {
        self.no_proxy = true;
        self.refresh_transport();
        self
    }

    /// Accept invalid TLS certificates (**dangerous**).
    pub fn danger_accept_invalid_certs(mut self, yes: bool) -> Self {
        self.insecure = yes;
        self.refresh_transport();
        self
    }

    /// Adjust the per-request timeout.
    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self.refresh_transport();
        self
    }
}

impl<T: AsyncTransport> JenkinsAsyncBuilder<T> {
    /// Apply HTTP basic authentication credentials.
    pub fn auth_basic(mut self, user: impl Into<String>, token: impl Into<String>) -> Self {
        self.auth = Some((user.into(), token.into()));
        self
    }

    /// Swap out the underlying transport implementation.
    pub fn transport<NT: AsyncTransport>(self, transport: NT) -> JenkinsAsyncBuilder<NT> {
        JenkinsAsyncBuilder {
            base_url: self.base_url,
            auth: self.auth,
            insecure: self.insecure,
            timeout: self.timeout,
            no_proxy: self.no_proxy,
            transport,
        }
    }

    /// Wrap the transport with a retry middleware.
    pub fn with_retry(self, max: usize, backoff: Duration) -> JenkinsAsyncBuilder<RetryAsync<T>> {
        let JenkinsAsyncBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport,
        } = self;

        JenkinsAsyncBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport: RetryAsync::new(transport, max, backoff),
        }
    }

    /// Wrap the transport with a crumb-fetching middleware.
    pub fn with_crumb(
        self,
        ttl: Duration,
    ) -> Result<JenkinsAsyncBuilder<CrumbAsync<T>>, JenkinsError> {
        let JenkinsAsyncBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport,
        } = self;

        let base_url_url = normalize_base_url(&base_url)?;
        let crumb_auth = auth.clone();

        Ok(JenkinsAsyncBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport: CrumbAsync::new(transport, base_url_url, crumb_auth, ttl, timeout),
        })
    }

    /// Finalise configuration and build the client.
    pub fn build(self) -> Result<JenkinsAsync<T>, JenkinsError> {
        let base = normalize_base_url(&self.base_url)?;
        Ok(JenkinsAsync {
            base,
            auth: self.auth,
            timeout: self.timeout,
            transport: self.transport,
        })
    }
}

#[derive(Clone)]
pub struct JenkinsAsync<T: AsyncTransport = DefaultAsyncTransport> {
    base: Url,
    auth: Option<(String, String)>,
    timeout: Duration,
    transport: T,
}

impl JenkinsAsync<DefaultAsyncTransport> {
    #[must_use]
    pub fn builder(base: impl Into<String>) -> JenkinsAsyncBuilder<DefaultAsyncTransport> {
        JenkinsAsyncBuilder::default_builder(base)
    }

    pub fn new(base: impl Into<String>) -> Result<Self, JenkinsError> {
        Self::builder(base).build()
    }
}

impl<T: AsyncTransport> JenkinsAsync<T> {
    fn own_pairs(v: Vec<(Cow<'_, str>, Cow<'_, str>)>) -> Vec<(String, String)> {
        v.into_iter()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    pub async fn request<E: Endpoint>(&self, ep: &E) -> Result<E::Output, JenkinsError> {
        let mut headers = HashMap::new();
        if let Some((user, token)) = &self.auth {
            headers.insert(
                "Authorization".into(),
                format!("Basic {}", B64.encode(format!("{user}:{token}"))),
            );
        }

        let params = ep.params().unwrap_or_default();
        let (query_raw, form_raw) = if ep.method() == Method::GET {
            (params, Vec::new())
        } else {
            (Vec::new(), params)
        };
        let (query, form) = (Self::own_pairs(query_raw), Self::own_pairs(form_raw));

        let url = self.base.join(&ep.path())?;
        let (status, body) = self
            .transport
            .send(ep.method(), url.clone(), headers, query, form, self.timeout)
            .await?;

        if !status.is_success() {
            return Err(JenkinsError::Http {
                code: status,
                method: ep.method(),
                url,
                body,
            });
        }

        ep.parse(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ConsoleText, QueueLength};
    use async_trait::async_trait;
    use http::{Method, StatusCode};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::Duration,
    };
    use url::Url;

    #[derive(Clone)]
    struct FakeTransport {
        status: StatusCode,
        body: String,
    }

    impl FakeTransport {
        fn new(status: StatusCode, body: impl Into<String>) -> Self {
            Self {
                status,
                body: body.into(),
            }
        }
    }

    #[async_trait]
    impl AsyncTransport for FakeTransport {
        async fn send(
            &self,
            _method: Method,
            _url: Url,
            _headers: HashMap<String, String>,
            _query: Vec<(String, String)>,
            _form: Vec<(String, String)>,
            _timeout: Duration,
        ) -> Result<(StatusCode, String), JenkinsError> {
            Ok((self.status, self.body.clone()))
        }
    }

    #[derive(Clone)]
    struct CapturingTransport {
        status: StatusCode,
        body: String,
        last_url: Arc<Mutex<Option<Url>>>,
    }

    impl CapturingTransport {
        fn new(status: StatusCode, body: impl Into<String>) -> Self {
            Self {
                status,
                body: body.into(),
                last_url: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[async_trait]
    impl AsyncTransport for CapturingTransport {
        async fn send(
            &self,
            _method: Method,
            url: Url,
            _headers: HashMap<String, String>,
            _query: Vec<(String, String)>,
            _form: Vec<(String, String)>,
            _timeout: Duration,
        ) -> Result<(StatusCode, String), JenkinsError> {
            *self.last_url.lock().unwrap() = Some(url.clone());
            Ok((self.status, self.body.clone()))
        }
    }

    #[test]
    fn build_rejects_invalid_url() {
        let result = JenkinsAsync::builder("not a url").build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn request_keeps_plain_text_bodies() {
        let client = JenkinsAsync::builder("https://example.com")
            .transport(FakeTransport::new(StatusCode::OK, "plain"))
            .build()
            .unwrap();

        let body: String = client.request(&ConsoleText("job", "42")).await.unwrap();
        assert_eq!(body, "plain");
    }

    #[tokio::test]
    async fn request_parses_json_by_default() {
        let client = JenkinsAsync::builder("https://example.com")
            .transport(FakeTransport::new(StatusCode::OK, r#"{ "items": [] }"#))
            .build()
            .unwrap();

        let json = client.request(&QueueLength).await.unwrap();
        assert!(json["items"].is_array());
    }

    #[tokio::test]
    async fn request_respects_base_path_without_trailing_slash() {
        let transport = CapturingTransport::new(StatusCode::OK, "{}");
        let last_url = transport.last_url.clone();

        let client = JenkinsAsync::builder("https://example.com/jenkins")
            .transport(transport)
            .build()
            .unwrap();

        let _ = client.request(&QueueLength).await.unwrap();

        let url = last_url.lock().unwrap().clone().unwrap();
        assert_eq!(url.as_str(), "https://example.com/jenkins/queue/api/json");
    }
}
