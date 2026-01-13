use super::{ResponseMeta, TransportRequest, TransportResponse};
use crate::error::{Error, TransportErrorKind};
use http::Method;
use std::{sync::Arc, time::Duration};
use ureq::Agent;

/// Trait implemented by any blocking HTTP layer.
pub trait BlockingTransport: Send + Sync + 'static {
    fn send(&self, req: TransportRequest) -> Result<TransportResponse, Error>;
}

pub type DynBlockingTransport = Arc<dyn BlockingTransport>;

impl<T: BlockingTransport + ?Sized> BlockingTransport for Arc<T> {
    fn send(&self, req: TransportRequest) -> Result<TransportResponse, Error> {
        (**self).send(req)
    }
}

/// Default blocking transport built on `ureq`.
#[derive(Clone)]
pub struct UreqBlocking {
    agent: Agent,
}

impl UreqBlocking {
    /// Construct a new transport.
    ///
    /// * See [`crate::transport::async_transport::ReqwestAsync::try_new`] for parameter meaning.
    pub fn try_new(
        insecure: bool,
        ua: &str,
        timeout: Duration,
        connect_timeout: Duration,
        read_timeout: Duration,
        no_proxy: bool,
    ) -> Result<Self, Error> {
        let mut builder = Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(timeout))
            .timeout_connect(Some(connect_timeout))
            .timeout_recv_body(Some(read_timeout))
            .user_agent(ua);

        if no_proxy {
            builder = builder.proxy(None);
        }

        if insecure {
            builder = builder.tls_config(
                ureq::tls::TlsConfig::builder()
                    .disable_verification(true)
                    .build(),
            );
        }

        Ok(Self {
            agent: Agent::new_with_config(builder.build()),
        })
    }
}

impl BlockingTransport for UreqBlocking {
    fn send(&self, req: TransportRequest) -> Result<TransportResponse, Error> {
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
        let url = url.as_str();
        let method_for_error = method.clone();

        let map_err = |err: ureq::Error| {
            let kind = match &err {
                ureq::Error::Timeout(_) => TransportErrorKind::Timeout,
                ureq::Error::HostNotFound | ureq::Error::ConnectionFailed => {
                    TransportErrorKind::Connect
                }
                ureq::Error::Io(io) if io.kind() == std::io::ErrorKind::TimedOut => {
                    TransportErrorKind::Timeout
                }
                ureq::Error::Io(io)
                    if matches!(
                        io.kind(),
                        std::io::ErrorKind::ConnectionRefused
                            | std::io::ErrorKind::ConnectionReset
                            | std::io::ErrorKind::ConnectionAborted
                            | std::io::ErrorKind::NotConnected
                    ) =>
                {
                    TransportErrorKind::Connect
                }
                _ => TransportErrorKind::Other,
            };

            Error::Transport {
                method: method_for_error.clone(),
                path: path.clone(),
                kind,
                source: Box::new(err),
            }
        };

        let mut response = match method {
            Method::GET => {
                drop(form);
                drop(body);
                let mut req = self.agent.get(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                req.config()
                    .timeout_global(Some(timeout))
                    .build()
                    .call()
                    .map_err(map_err)?
            }
            Method::DELETE => {
                drop(form);
                drop(body);
                let mut req = self.agent.delete(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                req.config()
                    .timeout_global(Some(timeout))
                    .build()
                    .call()
                    .map_err(map_err)?
            }
            Method::HEAD => {
                drop(form);
                drop(body);
                let mut req = self.agent.head(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                req.config()
                    .timeout_global(Some(timeout))
                    .build()
                    .call()
                    .map_err(map_err)?
            }
            Method::OPTIONS => {
                drop(form);
                drop(body);
                let mut req = self.agent.options(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                req.config()
                    .timeout_global(Some(timeout))
                    .build()
                    .call()
                    .map_err(map_err)?
            }
            Method::CONNECT => {
                drop(form);
                drop(body);
                let mut req = self.agent.connect(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                req.config()
                    .timeout_global(Some(timeout))
                    .build()
                    .call()
                    .map_err(map_err)?
            }
            Method::TRACE => {
                drop(form);
                drop(body);
                let mut req = self.agent.trace(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                req.config()
                    .timeout_global(Some(timeout))
                    .build()
                    .call()
                    .map_err(map_err)?
            }
            Method::POST => {
                let mut req = self.agent.post(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                let req = req.config().timeout_global(Some(timeout)).build();
                if let Some(body) = body {
                    let req = match body.content_type {
                        Some(content_type) => req.header(http::header::CONTENT_TYPE, content_type),
                        None => req,
                    };
                    req.send(body.bytes).map_err(map_err)?
                } else if form.is_empty() {
                    req.send_empty().map_err(map_err)?
                } else {
                    req.send_form(form).map_err(map_err)?
                }
            }
            Method::PUT => {
                let mut req = self.agent.put(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                let req = req.config().timeout_global(Some(timeout)).build();
                if let Some(body) = body {
                    let req = match body.content_type {
                        Some(content_type) => req.header(http::header::CONTENT_TYPE, content_type),
                        None => req,
                    };
                    req.send(body.bytes).map_err(map_err)?
                } else if form.is_empty() {
                    req.send_empty().map_err(map_err)?
                } else {
                    req.send_form(form).map_err(map_err)?
                }
            }
            Method::PATCH => {
                let mut req = self.agent.patch(url).query_pairs(query);
                for (name, value) in headers.iter() {
                    req = req.header(name, value);
                }
                let req = req.config().timeout_global(Some(timeout)).build();
                if let Some(body) = body {
                    let req = match body.content_type {
                        Some(content_type) => req.header(http::header::CONTENT_TYPE, content_type),
                        None => req,
                    };
                    req.send(body.bytes).map_err(map_err)?
                } else if form.is_empty() {
                    req.send_empty().map_err(map_err)?
                } else {
                    req.send_form(form).map_err(map_err)?
                }
            }
            other => {
                return Err(Error::InvalidConfig {
                    message: format!("unsupported HTTP method for blocking client: {other}")
                        .into_boxed_str(),
                    source: None,
                });
            }
        };

        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .body_mut()
            .with_config()
            .limit(u64::MAX)
            .read_to_vec()
            .map_err(map_err)?;

        Ok(TransportResponse {
            status,
            headers,
            body: body.to_vec(),
            meta: ResponseMeta::default(),
        })
    }
}
