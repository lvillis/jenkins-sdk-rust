//! Blocking CSRF-Crumb middleware.

use crate::{core::error::JenkinsError, transport::blocking_impl::BlockingTransport};
use base64::Engine;
use http::{Method, StatusCode};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use url::Url;

#[derive(Deserialize)]
struct CrumbResp {
    #[serde(rename = "crumbRequestField")]
    field: String,
    crumb: String,
}

struct CachedCrumb {
    field: String,
    crumb: String,
    ts: Instant,
}

/// Blocking wrapper that injects a crumb header.
#[derive(Clone)]
pub struct CrumbBlocking<T> {
    inner: T,
    base_url: Url,
    auth_basic: Option<(String, String)>,
    ttl: Duration,
    cache: Arc<Mutex<Option<CachedCrumb>>>,
}

impl<T: BlockingTransport> CrumbBlocking<T> {
    pub fn new(
        inner: T,
        base_url: Url,
        auth_basic: Option<(String, String)>,
        ttl: Duration,
    ) -> Self {
        Self {
            inner,
            base_url,
            auth_basic,
            ttl,
            cache: Arc::new(Mutex::new(None)),
        }
    }

    fn fetch_crumb(&self) -> Result<CachedCrumb, JenkinsError> {
        let url = self.base_url.join("crumbIssuer/api/json")?;
        let url_for_error = url.clone();

        let mut hdrs = HashMap::new();
        if let Some((u, p)) = &self.auth_basic {
            hdrs.insert(
                "Authorization".into(),
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD.encode(format!("{u}:{p}"))
                ),
            );
        }

        let (code, body) = self.inner.send(
            Method::GET,
            url,
            hdrs,
            vec![],
            vec![],
            Duration::from_secs(30),
        )?;

        if !code.is_success() {
            return Err(JenkinsError::Http {
                code,
                method: Method::GET,
                url: url_for_error,
                body,
            });
        }

        let json: CrumbResp = serde_json::from_str(&body)?;
        Ok(CachedCrumb {
            field: json.field,
            crumb: json.crumb,
            ts: Instant::now(),
        })
    }
}

impl<T: BlockingTransport> BlockingTransport for CrumbBlocking<T> {
    fn send(
        &self,
        method: Method,
        url: Url,
        mut headers: HashMap<String, String>,
        query: Vec<(String, String)>,
        form: Vec<(String, String)>,
        timeout: Duration,
    ) -> Result<(StatusCode, String), JenkinsError> {
        if method != Method::GET {
            let mut guard = self.cache.lock().unwrap();
            let expired = guard
                .as_ref()
                .map(|c| c.ts.elapsed() > self.ttl)
                .unwrap_or(true);

            if expired {
                *guard = Some(self.fetch_crumb()?);
            }
            if let Some(c) = guard.as_ref() {
                headers.insert(c.field.clone(), c.crumb.clone());
            }
        }

        self.inner.send(method, url, headers, query, form, timeout)
    }
}
