//! Reqwest-based transport layers.
//!
//! * `ReqwestAsync` / `ReqwestBlocking` respect `no_proxy` flag to ignore
//!   system proxy environment variables (HTTP_PROXY, HTTPS_PROXY, etc.).
//! * Both enable cookie-store so that JSESSIONID persists for CSRF crumbs.

use crate::core::error::JenkinsError;
use http::{Method, StatusCode};
use std::{collections::HashMap, time::Duration};
use url::Url;

/* ─────────── async client ─────────── */
#[cfg(feature = "async-client")]
pub mod async_impl {
    use super::*;
    use async_trait::async_trait;
    use reqwest::Client;

    /// Trait implemented by any async HTTP layer.
    #[async_trait]
    pub trait AsyncTransport: Clone + Send + Sync + 'static {
        async fn send(
            &self,
            method: Method,
            url: Url,
            headers: HashMap<String, String>,
            query: Vec<(String, String)>,
            form: Vec<(String, String)>,
            timeout: Duration,
        ) -> Result<(StatusCode, String), JenkinsError>;
    }

    /// Default async transport built on `reqwest`.
    #[derive(Clone)]
    pub struct ReqwestAsync {
        client: Client,
    }

    impl ReqwestAsync {
        /// Construct a new transport.
        ///
        /// * `insecure` – accept invalid TLS certificates.  
        /// * `ua` – User-Agent header.  
        /// * `timeout` – per-request timeout.  
        /// * `no_proxy` – ignore system proxy environment variables.
        pub fn new(insecure: bool, ua: &str, timeout: Duration, no_proxy: bool) -> Self {
            let mut builder = Client::builder()
                .danger_accept_invalid_certs(insecure)
                .user_agent(ua)
                .cookie_store(true)
                .timeout(timeout);

            if no_proxy {
                builder = builder.no_proxy();
            }

            Self {
                client: builder.build().expect("build reqwest"),
            }
        }
    }

    #[async_trait]
    impl AsyncTransport for ReqwestAsync {
        async fn send(
            &self,
            method: Method,
            url: Url,
            headers: HashMap<String, String>,
            query: Vec<(String, String)>,
            form: Vec<(String, String)>,
            timeout: Duration,
        ) -> Result<(StatusCode, String), JenkinsError> {
            let mut req = self
                .client
                .request(method.clone(), url.clone())
                .query(&query)
                .timeout(timeout);

            for (k, v) in &headers {
                req = req.header(k, v);
            }
            if !form.is_empty() {
                req = req.form(&form);
            }

            let resp = req.send().await.map_err(|e| JenkinsError::Reqwest {
                source: e,
                method: method.clone(),
                url: url.clone(),
            })?;

            let code = resp.status();
            let body = resp.text().await.map_err(|e| JenkinsError::Reqwest {
                source: e,
                method: method.clone(),
                url: url.clone(),
            })?;
            Ok((code, body))
        }
    }

    pub type DefaultAsyncTransport = ReqwestAsync;
}

/* ─────────── blocking client ─────────── */
#[cfg(feature = "blocking-client")]
pub mod blocking_impl {
    use super::*;
    use reqwest::blocking::Client;

    /// Trait implemented by any blocking HTTP layer.
    pub trait BlockingTransport: Clone + Send + Sync + 'static {
        fn send(
            &self,
            method: Method,
            url: Url,
            headers: HashMap<String, String>,
            query: Vec<(String, String)>,
            form: Vec<(String, String)>,
            timeout: Duration,
        ) -> Result<(StatusCode, String), JenkinsError>;
    }

    /// Default blocking transport built on `reqwest`.
    #[derive(Clone)]
    pub struct ReqwestBlocking {
        client: Client,
    }

    impl ReqwestBlocking {
        /// Construct a new transport.
        ///
        /// * See [`async_impl::ReqwestAsync::new`] for parameter meaning.
        pub fn new(insecure: bool, ua: &str, timeout: Duration, no_proxy: bool) -> Self {
            let mut builder = Client::builder()
                .danger_accept_invalid_certs(insecure)
                .user_agent(ua)
                .cookie_store(true)
                .timeout(timeout);

            if no_proxy {
                builder = builder.no_proxy();
            }

            Self {
                client: builder.build().expect("build reqwest"),
            }
        }
    }

    impl BlockingTransport for ReqwestBlocking {
        fn send(
            &self,
            method: Method,
            url: Url,
            headers: HashMap<String, String>,
            query: Vec<(String, String)>,
            form: Vec<(String, String)>,
            timeout: Duration,
        ) -> Result<(StatusCode, String), JenkinsError> {
            let mut req = self
                .client
                .request(method.clone(), url.clone())
                .query(&query)
                .timeout(timeout);

            for (k, v) in &headers {
                req = req.header(k, v);
            }
            if !form.is_empty() {
                req = req.form(&form);
            }

            let resp = req.send().map_err(|e| JenkinsError::Reqwest {
                source: e,
                method: method.clone(),
                url: url.clone(),
            })?;

            let code = resp.status();
            let body = resp.text().map_err(|e| JenkinsError::Reqwest {
                source: e,
                method,
                url,
            })?;
            Ok((code, body))
        }
    }

    pub type DefaultBlockingTransport = ReqwestBlocking;
}
