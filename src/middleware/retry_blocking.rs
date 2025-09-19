//! Exponential back-off retry wrapper (blocking).

use crate::{core::error::JenkinsError, transport::blocking_impl::BlockingTransport};
use http::{Method, StatusCode};
use std::{collections::HashMap, thread::sleep, time::Duration};
use url::Url;

/// Retry wrapper for blocking transports.
#[derive(Clone)]
pub struct RetryBlocking<T> {
    inner: T,
    max: usize,
    base_delay: Duration,
}

impl<T> RetryBlocking<T> {
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

impl<T: BlockingTransport> BlockingTransport for RetryBlocking<T> {
    fn send(
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
            let (code, body) = self.inner.send(
                method.clone(),
                url.clone(),
                headers.clone(),
                query.clone(),
                form.clone(),
                timeout,
            )?;

            if code.is_server_error() && attempt < self.max {
                attempt += 1;
                let delay = self.delay_for(attempt);
                if !delay.is_zero() {
                    sleep(delay);
                }
                continue;
            }
            return Ok((code, body));
        }
    }
}
