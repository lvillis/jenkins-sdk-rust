//! Exponential back-off retry wrapper (blocking).

use crate::{core::error::JenkinsError, transport::blocking_impl::BlockingTransport};
use http::{Method, StatusCode};
use std::{collections::HashMap, thread, time::Duration};
use url::Url;

/// Retry wrapper for blocking transports.
#[derive(Clone)]
pub struct RetryBlocking<T> {
    inner: T,
    max: usize,
    backoff: Duration,
}

impl<T> RetryBlocking<T> {
    /// Create a new retry layer.
    ///
    /// * `inner`  – the wrapped transport  
    /// * `max`    – maximum retry attempts ( ≥ 1 )  
    /// * `backoff`– initial back-off duration
    pub fn new(inner: T, max: usize, backoff: Duration) -> Self {
        Self {
            inner,
            max,
            backoff,
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

            // Retry on 5xx up to `max` attempts.
            if code.is_server_error() && attempt < self.max {
                attempt += 1;

                // Exponential back-off: base * 2^attempt
                let delay = self.backoff.mul_f64(2f64.powi(attempt as i32)); // e.g. 200 ms → 400 ms → 800 ms …

                thread::sleep(delay);
                continue;
            }
            return Ok((code, body));
        }
    }
}
