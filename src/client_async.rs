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
/// *Do not construct directly; use* `JenkinsAsync::builder(..)` *instead.*
pub struct JenkinsAsyncBuilder<T = DefaultAsyncTransport> {
    base_url: String,
    auth: Option<(String, String)>,
    insecure: bool,
    timeout: Duration,
    no_proxy: bool,
    transport: T,
}

/* ───────────── internal constructor ───────────── */
impl JenkinsAsyncBuilder<DefaultAsyncTransport> {
    /// Internal helper used by `JenkinsAsync::builder`.
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

    /// Ignore proxy environment variables.
    pub fn no_system_proxy(mut self) -> Self {
        self.no_proxy = true;
        self.transport =
            DefaultAsyncTransport::new(self.insecure, "jenkins-sdk-rust", self.timeout, true);
        self
    }
}

/* ───────────── generic part ───────────── */
impl<T: AsyncTransport> JenkinsAsyncBuilder<T> {
    /* setters */
    pub fn auth_basic(mut self, user: impl Into<String>, token: impl Into<String>) -> Self {
        self.auth = Some((user.into(), token.into()));
        self
    }
    pub fn danger_accept_invalid_certs(mut self, yes: bool) -> Self {
        self.insecure = yes;
        self
    }
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }

    /// Replace the current transport instance.
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

    /* sugar */
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

    /// Finalize the builder and create a client.
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

    /// Build a client with all default settings (quick path).
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

        let (code, body) = self
            .transport
            .send(
                ep.method(),
                self.base.join(&ep.path())?,
                hdr,
                query,
                form,
                self.timeout,
            )
            .await?;

        if !code.is_success() {
            return Err(JenkinsError::Http { code, body });
        }

        /* decode */
        if TypeId::of::<E::Output>() == TypeId::of::<String>() {
            let boxed: Box<dyn Any> = Box::new(body);
            return Ok(*boxed.downcast::<E::Output>().unwrap());
        }

        Ok(serde_json::from_str(&body)?)
    }
}
