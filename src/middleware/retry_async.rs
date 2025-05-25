//! Exponential back-off retry wrapper (async).

use crate::{core::error::JenkinsError, transport::async_impl::AsyncTransport};
use async_trait::async_trait;
use http::{Method, StatusCode};
use std::{collections::HashMap, time::Duration};
use url::Url;

/// Retry wrapper for async transports.
#[derive(Clone)]
pub struct RetryAsync<T> {
    inner: T,
    max: usize,
    backoff: Duration,
}

impl<T> RetryAsync<T> {
    pub fn new(inner: T, max: usize, backoff: Duration) -> Self {
        Self {
            inner,
            max,
            backoff,
        }
    }
}

#[async_trait]
impl<T: AsyncTransport> AsyncTransport for RetryAsync<T> {
    async fn send(
        &self,
        method: Method,
        url: Url,
        headers: HashMap<String, String>,
        query: Vec<(String, String)>,
        form: Vec<(String, String)>,
        timeout: Duration,
    ) -> Result<(StatusCode, String), JenkinsError> {
        let mut attempt = 0;

        loop {
            let (code, body) = self
                .inner
                .send(
                    method.clone(),
                    url.clone(),
                    headers.clone(),
                    query.clone(),
                    form.clone(),
                    timeout,
                )
                .await?;

            if code.is_server_error() && attempt < self.max {
                attempt += 1;
                let delay = self.backoff.mul_f64(attempt as f64); // <-- fixed
                tokio::time::sleep(delay).await;
                continue;
            }
            return Ok((code, body));
        }
    }
}
