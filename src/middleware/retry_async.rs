//! Exponential back-off retry wrapper (async).

use crate::{core::error::JenkinsError, transport::async_impl::AsyncTransport};
use async_trait::async_trait;
use http::{Method, StatusCode};
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;
use url::Url;

/// Retry wrapper for async transports.
#[derive(Clone)]
pub struct RetryAsync<T> {
    inner: T,
    max: usize,
    base_delay: Duration,
}

impl<T> RetryAsync<T> {
    /// Create a new retry layer.
    ///
    /// * inner  - the wrapped transport
    /// * max    - maximum retry attempts (>= 1)
    /// * base   - base delay used for exponential back-off
    pub fn new(inner: T, max: usize, base: Duration) -> Self {
        Self {
            inner,
            max,
            base_delay: base,
        }
    }

    fn delay_for(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            Duration::from_secs(0)
        } else {
            self.base_delay.mul_f64(2f64.powi((attempt - 1) as i32))
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
                let delay = self.delay_for(attempt);
                if !delay.is_zero() {
                    sleep(delay).await;
                }
                continue;
            }
            return Ok((code, body));
        }
    }
}
