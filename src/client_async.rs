//! High-level asynchronous Jenkins client.

use crate::{
    core::{Endpoint, JenkinsError},
    middleware::{Crumb, Retry},
    transport::async_impl::{AsyncTransport, DefaultAsyncTransport},
};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use http::Method;
use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    time::Duration,
};
use url::Url;

/// Builder for [`JenkinsAsync`].
///
/// *Do not construct directly; call* `JenkinsAsync::builder(..)` *instead.*
pub struct JenkinsAsyncBuilder<T = DefaultAsyncTransport> {
    base_url: String,
    auth: Option<(String, String)>,
    insecure: bool,
    timeout: Duration,
    no_proxy: bool,
    transport: T,
}

/* ───────────── impl for DefaultAsyncTransport ───────────── */
impl JenkinsAsyncBuilder<DefaultAsyncTransport> {
    /// Create a builder with default settings.
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

    /// Rebuild the internal default transport after flag changes.
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

    /// Set the per-request timeout.
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self.refresh_transport();
        self
    }
}

/* ───────────── generic impl (any transport) ───────────── */
impl<T: AsyncTransport> JenkinsAsyncBuilder<T> {
    /* setters that do not require rebuilding default transport */
    pub fn auth_basic(mut self, user: impl Into<String>, token: impl Into<String>) -> Self {
        self.auth = Some((user.into(), token.into()));
        self
    }

    /// Swap out the underlying transport.
    pub fn transport<NT: AsyncTransport>(self, t: NT) -> JenkinsAsyncBuilder<NT> {
        JenkinsAsyncBuilder {
            base_url: self.base_url,
            auth: self.auth,
            insecure: self.insecure,
            timeout: self.timeout,
            no_proxy: self.no_proxy,
            transport: t,
        }
    }

    /* sugar layers */
    pub fn with_retry(self, max: usize, backoff: Duration) -> JenkinsAsyncBuilder<Retry<T>> {
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
            transport: Retry::new(transport, max, backoff),
        }
    }

    pub fn with_crumb(self, ttl: Duration) -> JenkinsAsyncBuilder<Crumb<T>> {
        let JenkinsAsyncBuilder {
            base_url,
            auth,
            insecure,
            timeout,
            no_proxy,
            transport,
        } = self;

        let base_url_url = Url::parse(&base_url).expect("valid base_url");

        JenkinsAsyncBuilder {
            base_url,
            auth: auth.clone(),
            insecure,
            timeout,
            no_proxy,
            transport: Crumb::new(transport, base_url_url, auth, ttl),
        }
    }

    /// Finalize the builder and create the client.
    pub fn build(self) -> JenkinsAsync<T> {
        JenkinsAsync {
            base: Url::parse(&self.base_url).expect("valid URL"),
            auth: self.auth,
            timeout: self.timeout,
            transport: self.transport,
        }
    }
}

/* ───────────── concrete client ───────────── */
#[derive(Clone)]
pub struct JenkinsAsync<T: AsyncTransport = DefaultAsyncTransport> {
    base: Url,
    auth: Option<(String, String)>,
    timeout: Duration,
    transport: T,
}

impl JenkinsAsync<DefaultAsyncTransport> {
    /// Start a builder chain (recommended).
    #[must_use]
    pub fn builder(base: impl Into<String>) -> JenkinsAsyncBuilder<DefaultAsyncTransport> {
        JenkinsAsyncBuilder::default_builder(base)
    }

    /// Quick path: all default settings.
    #[must_use]
    pub fn new(base: impl Into<String>) -> Self {
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
        /* headers */
        let mut hdr = HashMap::new();
        if let Some((u, p)) = &self.auth {
            hdr.insert(
                "Authorization".into(),
                format!("Basic {}", B64.encode(format!("{u}:{p}"))),
            );
        }

        /* params */
        let params = ep.params().unwrap_or_default();
        let (q_raw, f_raw) = if ep.method() == Method::GET {
            (params, Vec::new())
        } else {
            (Vec::new(), params)
        };
        let (query, form) = (Self::own_pairs(q_raw), Self::own_pairs(f_raw));

        let url = self.base.join(&ep.path())?;

        let (code, body) = self
            .transport
            .send(ep.method(), url.clone(), hdr, query, form, self.timeout)
            .await?;

        if !code.is_success() {
            return Err(JenkinsError::Http {
                code,
                method: ep.method(),
                url,
                body,
            });
        }

        /* decode */
        if TypeId::of::<E::Output>() == TypeId::of::<String>() {
            let boxed: Box<dyn Any> = Box::new(body);
            return Ok(*boxed.downcast::<E::Output>().unwrap());
        }

        Ok(serde_json::from_str(&body)?)
    }
}
