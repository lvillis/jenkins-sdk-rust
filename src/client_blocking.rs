//! High-level blocking Jenkins client.

use crate::{
    core::{Endpoint, JenkinsError},
    middleware::{CrumbBlocking, RetryBlocking},
    transport::blocking_impl::{BlockingTransport, DefaultBlockingTransport},
};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use http::Method;
use std::{borrow::Cow, collections::HashMap, time::Duration};
use url::Url;

/// Configures and constructs [`JenkinsBlocking`].
pub struct JenkinsBlockingBuilder<T = DefaultBlockingTransport> {
    base_url: String,
    auth: Option<(String, String)>,
    insecure: bool,
    timeout: Duration,
    no_proxy: bool,
    transport: T,
}

impl JenkinsBlockingBuilder<DefaultBlockingTransport> {
    /// Create a builder with opinionated defaults.
    fn default_builder(base: impl Into<String>) -> Self {
        Self {
            base_url: base.into(),
            auth: None,
            insecure: false,
            timeout: Duration::from_secs(30),
            no_proxy: false,
            transport: DefaultBlockingTransport::new(
                false,
                "jenkins-sdk-rust",
                Duration::from_secs(30),
                false,
            ),
        }
    }

    /// Rebuild the transport when default flags change.
    fn refresh_transport(&mut self) {
        self.transport = DefaultBlockingTransport::new(
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

impl<T: BlockingTransport> JenkinsBlockingBuilder<T> {
    /// Apply HTTP basic authentication credentials.
    pub fn auth_basic(mut self, user: impl Into<String>, token: impl Into<String>) -> Self {
        self.auth = Some((user.into(), token.into()));
        self
    }

    /// Swap out the underlying transport implementation.
    pub fn transport<NT: BlockingTransport>(self, transport: NT) -> JenkinsBlockingBuilder<NT> {
        JenkinsBlockingBuilder {
            base_url: self.base_url,
            auth: self.auth,
            insecure: self.insecure,
            timeout: self.timeout,
            no_proxy: self.no_proxy,
            transport,
        }
    }

    /// Wrap the transport with a retry middleware.
    pub fn with_retry(
        self,
        max: usize,
        backoff: Duration,
    ) -> JenkinsBlockingBuilder<RetryBlocking<T>> {
        let JenkinsBlockingBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport,
        } = self;

        JenkinsBlockingBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport: RetryBlocking::new(transport, max, backoff),
        }
    }

    /// Wrap the transport with a crumb-fetching middleware.
    pub fn with_crumb(
        self,
        ttl: Duration,
    ) -> Result<JenkinsBlockingBuilder<CrumbBlocking<T>>, JenkinsError> {
        let JenkinsBlockingBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport,
        } = self;

        let base_url_url = Url::parse(&base_url)?;
        let crumb_auth = auth.clone();

        Ok(JenkinsBlockingBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport: CrumbBlocking::new(transport, base_url_url, crumb_auth, ttl, timeout),
        })
    }

    /// Finalise configuration and build the client.
    pub fn build(self) -> Result<JenkinsBlocking<T>, JenkinsError> {
        let base = Url::parse(&self.base_url)?;
        Ok(JenkinsBlocking {
            base,
            auth: self.auth,
            timeout: self.timeout,
            transport: self.transport,
        })
    }
}

#[derive(Clone)]
pub struct JenkinsBlocking<T: BlockingTransport = DefaultBlockingTransport> {
    base: Url,
    auth: Option<(String, String)>,
    timeout: Duration,
    transport: T,
}

impl JenkinsBlocking<DefaultBlockingTransport> {
    #[must_use]
    pub fn builder(base: impl Into<String>) -> JenkinsBlockingBuilder<DefaultBlockingTransport> {
        JenkinsBlockingBuilder::default_builder(base)
    }

    pub fn new(base: impl Into<String>) -> Result<Self, JenkinsError> {
        Self::builder(base).build()
    }
}

impl<T: BlockingTransport> JenkinsBlocking<T> {
    fn own_pairs(v: Vec<(Cow<'_, str>, Cow<'_, str>)>) -> Vec<(String, String)> {
        v.into_iter()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    pub fn request<E: Endpoint>(&self, ep: &E) -> Result<E::Output, JenkinsError> {
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

        let mut url = self.base.clone();
        {
            let mut url_path = url
                .path_segments_mut()
                .expect("Base URL should not be a cannot-be-a-base URL");

            for p in ep.path().split('/') {
                url_path.push(p);
            }
        }
        let url = url;

        let (status, body) =
            self.transport
                .send(ep.method(), url.clone(), headers, query, form, self.timeout)?;

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
    use http::{Method, StatusCode};
    use std::{collections::HashMap, time::Duration};
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

    impl BlockingTransport for FakeTransport {
        fn send(
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

    #[test]
    fn build_rejects_invalid_url() {
        let result = JenkinsBlocking::builder("not a url").build();
        assert!(result.is_err());
    }

    #[test]
    fn request_keeps_plain_text_bodies() {
        let client = JenkinsBlocking::builder("https://example.com")
            .transport(FakeTransport::new(StatusCode::OK, "plain"))
            .build()
            .unwrap();

        let body: String = client.request(&ConsoleText("job", "42")).unwrap();
        assert_eq!(body, "plain");
    }

    #[test]
    fn request_parses_json_by_default() {
        let client = JenkinsBlocking::builder("https://example.com")
            .transport(FakeTransport::new(StatusCode::OK, r#"{ "items": [] }"#))
            .build()
            .unwrap();

        let json = client.request(&QueueLength).unwrap();
        assert!(json["items"].is_array());
    }
}
