use super::{ResponseMeta, TransportRequest, TransportResponse};
use crate::error::{Error, TransportErrorKind};
use async_trait::async_trait;
use reqwest::Client;
use std::{sync::Arc, time::Duration};

#[cfg(feature = "rustls")]
fn ensure_rustls_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

#[cfg(not(feature = "rustls"))]
fn ensure_rustls_provider() {}

/// Trait implemented by any async HTTP layer.
#[async_trait]
pub trait AsyncTransport: Send + Sync + 'static {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, Error>;
}

pub type DynAsyncTransport = Arc<dyn AsyncTransport>;

#[async_trait]
impl<T: AsyncTransport + ?Sized> AsyncTransport for Arc<T> {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, Error> {
        (**self).send(req).await
    }
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
    /// * `connect_timeout` – connection establishment timeout.
    /// * `no_proxy` – ignore system proxy environment variables.
    pub fn try_new(
        insecure: bool,
        ua: &str,
        timeout: Duration,
        connect_timeout: Duration,
        no_proxy: bool,
    ) -> Result<Self, Error> {
        ensure_rustls_provider();

        let mut builder = Client::builder()
            .danger_accept_invalid_certs(insecure)
            .user_agent(ua)
            .cookie_store(true)
            .connect_timeout(connect_timeout)
            .timeout(timeout);

        if no_proxy {
            builder = builder.no_proxy();
        }

        let client = builder.build().map_err(|err| Error::InvalidConfig {
            message: "failed to build async HTTP client".into(),
            source: Some(Box::new(err)),
        })?;

        Ok(Self { client })
    }
}

#[async_trait]
impl AsyncTransport for ReqwestAsync {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, Error> {
        let TransportRequest {
            method,
            url,
            headers,
            query,
            form,
            body,
            timeout,
        } = req;
        let mut req = self
            .client
            .request(method.clone(), url.clone())
            .query(&query)
            .timeout(timeout);

        req = req.headers(headers);
        if let Some(body) = body {
            if let Some(content_type) = body.content_type {
                req = req.header(http::header::CONTENT_TYPE, content_type);
            }
            req = req.body(body.bytes);
        } else if !form.is_empty() {
            req = req.form(&form);
        }

        let resp = req.send().await.map_err(|e| {
            let kind = if e.is_timeout() {
                TransportErrorKind::Timeout
            } else if e.is_connect() {
                TransportErrorKind::Connect
            } else {
                TransportErrorKind::Other
            };
            Error::Transport {
                method: method.clone(),
                path: url.path().to_string().into_boxed_str(),
                kind,
                source: Box::new(e),
            }
        })?;

        let code = resp.status();
        let headers = resp.headers().clone();
        let body = resp.bytes().await.map_err(|e| {
            let kind = if e.is_timeout() {
                TransportErrorKind::Timeout
            } else if e.is_connect() {
                TransportErrorKind::Connect
            } else {
                TransportErrorKind::Other
            };
            Error::Transport {
                method: method.clone(),
                path: url.path().to_string().into_boxed_str(),
                kind,
                source: Box::new(e),
            }
        })?;
        Ok(TransportResponse {
            status: code,
            headers,
            body: body.to_vec(),
            meta: ResponseMeta::default(),
        })
    }
}
