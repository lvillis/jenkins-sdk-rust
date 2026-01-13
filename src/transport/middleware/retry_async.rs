//! Conservative retry wrapper (async).

use super::retry::{
    RetryConfig, backoff_delay, is_idempotent, is_retryable_status, jitter_delay, parse_retry_after,
};
use crate::{
    Error, TransportErrorKind,
    transport::{
        ResponseMeta, TransportRequest, TransportResponse,
        async_transport::{AsyncTransport, DynAsyncTransport},
    },
};
use async_trait::async_trait;
use http::Method;
use std::time::SystemTime;
use tokio::time::sleep;

#[derive(Clone)]
pub struct RetryAsync {
    inner: DynAsyncTransport,
    config: RetryConfig,
}

impl RetryAsync {
    #[must_use]
    pub fn new(inner: DynAsyncTransport, config: RetryConfig) -> Self {
        Self { inner, config }
    }

    fn should_retry_method(&self, method: &Method) -> bool {
        self.config.retry_non_idempotent || is_idempotent(method)
    }

    fn should_retry_error(&self, err: &Error) -> bool {
        matches!(err, Error::Transport { kind, .. } if matches!(kind, TransportErrorKind::Timeout | TransportErrorKind::Connect))
    }
}

#[async_trait]
impl AsyncTransport for RetryAsync {
    async fn send(&self, req: TransportRequest) -> Result<TransportResponse, Error> {
        let can_retry = self.should_retry_method(&req.method);

        let mut retries = 0usize;
        let request = req;
        loop {
            let result = self.inner.send(request.clone()).await;

            match result {
                Ok(mut resp) => {
                    if can_retry
                        && retries < self.config.max_retries
                        && is_retryable_status(resp.status)
                    {
                        let now = SystemTime::now();
                        let retry_after = self
                            .config
                            .respect_retry_after
                            .then(|| parse_retry_after(&resp.headers, now))
                            .flatten();

                        let delay = retry_after.unwrap_or_else(|| {
                            let cap = backoff_delay(&self.config, retries + 1);
                            if self.config.jitter {
                                jitter_delay(cap)
                            } else {
                                cap
                            }
                        });

                        if !delay.is_zero() {
                            sleep(delay).await;
                        }
                        retries += 1;
                        continue;
                    }

                    resp.meta = ResponseMeta {
                        retries: resp.meta.retries.saturating_add(retries),
                    };
                    return Ok(resp);
                }
                Err(err) => {
                    if can_retry
                        && retries < self.config.max_retries
                        && self.should_retry_error(&err)
                    {
                        let cap = backoff_delay(&self.config, retries + 1);
                        let delay = if self.config.jitter {
                            jitter_delay(cap)
                        } else {
                            cap
                        };
                        if !delay.is_zero() {
                            sleep(delay).await;
                        }
                        retries += 1;
                        continue;
                    }
                    return Err(err);
                }
            }
        }
    }
}
