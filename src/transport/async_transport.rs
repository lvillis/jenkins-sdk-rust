use super::{ResponseMeta, TransportRequest, TransportResponse};
use crate::{
    TlsRootStore,
    error::{Error, TransportErrorKind},
    util::proxy_env::load_proxy_env,
};
use async_trait::async_trait;
use reqx::{
    Client, Error as ReqxError, RetryPolicy, StatusPolicy,
    TransportErrorKind as ReqxTransportErrorKind,
};
use std::{sync::Arc, time::Duration};

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

/// Default async transport built on `reqx`.
#[derive(Clone)]
pub struct ReqxAsync {
    client: Client,
}

impl ReqxAsync {
    /// Construct a new transport.
    ///
    /// * `base_url` - client base URL used by reqx.
    /// * `ua` - User-Agent header.
    /// * `timeout` - per-request timeout.
    /// * `connect_timeout` – connection establishment timeout.
    /// * `disable_system_proxy` - disable system proxy environment variables.
    /// * `tls_root_store` - trust root selection for TLS.
    pub fn try_new(
        base_url: &str,
        ua: &str,
        timeout: Duration,
        connect_timeout: Duration,
        disable_system_proxy: bool,
        tls_root_store: TlsRootStore,
    ) -> Result<Self, Error> {
        let ua = http::HeaderValue::from_str(ua).map_err(|source| Error::InvalidConfig {
            message: "invalid User-Agent header value".into(),
            source: Some(Box::new(source)),
        })?;

        let proxy_env = load_proxy_env(base_url, disable_system_proxy)?;

        let mut builder = Client::builder(base_url)
            .request_timeout(timeout)
            .connect_timeout(connect_timeout)
            .retry_policy(RetryPolicy::disabled())
            .tls_root_store(tls_root_store.into_reqx())
            .default_header(http::header::USER_AGENT, ua);

        if let Some(proxy_uri) = proxy_env.proxy_uri {
            builder = builder.http_proxy(proxy_uri);
            if !proxy_env.no_proxy_rules.is_empty() {
                builder = builder
                    .try_no_proxy(&proxy_env.no_proxy_rules)
                    .map_err(|source| Error::InvalidConfig {
                        message: "invalid NO_PROXY/no_proxy rule".into(),
                        source: Some(Box::new(source)),
                    })?;
            }
        }

        let client = builder.build().map_err(|source| Error::InvalidConfig {
            message: "failed to build async HTTP client".into(),
            source: Some(Box::new(source)),
        })?;

        Ok(Self { client })
    }
}

fn map_reqx_error(err: ReqxError, method: http::Method, path: Box<str>) -> Error {
    let kind = match &err {
        ReqxError::Timeout { .. } | ReqxError::DeadlineExceeded { .. } => {
            TransportErrorKind::Timeout
        }
        ReqxError::Transport {
            kind: ReqxTransportErrorKind::Dns | ReqxTransportErrorKind::Connect,
            ..
        } => TransportErrorKind::Connect,
        ReqxError::Transport { .. } => TransportErrorKind::Other,
        _ => TransportErrorKind::Other,
    };

    Error::Transport {
        method,
        path,
        kind,
        source: Box::new(err),
    }
}

#[async_trait]
impl AsyncTransport for ReqxAsync {
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
        let path = url.path().to_string().into_boxed_str();

        let mut request = self
            .client
            .request(method.clone(), url.as_str().to_owned())
            .query_pairs(query)
            .timeout(timeout)
            .status_policy(StatusPolicy::Response);

        for (name, value) in headers.iter() {
            request = request.header(name.clone(), value.clone());
        }
        if let Some(body) = body {
            if let Some(content_type) = body.content_type {
                request = request.header(http::header::CONTENT_TYPE, content_type);
            }
            request = request.body(body.bytes);
        } else if !form.is_empty() {
            request = request
                .form(&form)
                .map_err(|err| map_reqx_error(err, method.clone(), path.clone()))?;
        }

        let resp = request
            .send()
            .await
            .map_err(|err| map_reqx_error(err, method, path))?;

        Ok(TransportResponse {
            status: resp.status(),
            headers: resp.headers().clone(),
            body: resp.body().to_vec(),
            meta: ResponseMeta::default(),
        })
    }
}
