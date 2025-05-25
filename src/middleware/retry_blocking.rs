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

            if code.is_server_error() && attempt < self.max {
                attempt += 1;
                let delay = self.backoff.mul_f64(attempt as f64); // <-- fixed
                thread::sleep(delay);
                continue;
            }
            return Ok((code, body));
        }
    }
}
