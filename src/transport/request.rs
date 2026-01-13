use http::{HeaderMap, HeaderValue, Method, StatusCode};
use serde::de::DeserializeOwned;
#[cfg(feature = "unstable-raw")]
use std::borrow::Cow;
use std::time::Duration;

#[cfg(feature = "unstable-raw")]
use http::HeaderName;

#[derive(Clone, Debug)]
pub struct RequestBody {
    pub bytes: Vec<u8>,
    pub content_type: Option<HeaderValue>,
}

impl RequestBody {
    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            content_type: None,
        }
    }

    #[must_use]
    pub fn bytes_with_content_type(bytes: Vec<u8>, content_type: HeaderValue) -> Self {
        Self {
            bytes,
            content_type: Some(content_type),
        }
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn text(text: impl Into<String>) -> Self {
        Self::bytes(text.into().into_bytes())
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn text_with_content_type(text: impl Into<String>, content_type: HeaderValue) -> Self {
        Self::bytes_with_content_type(text.into().into_bytes(), content_type)
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    pub segments: Vec<String>,
    pub query: Vec<(String, String)>,
    pub form: Vec<(String, String)>,
    pub headers: HeaderMap,
    pub body: Option<RequestBody>,
    pub timeout_override: Option<Duration>,
}

impl Request {
    #[must_use]
    pub fn new<I, S>(method: Method, segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            method,
            segments: segments.into_iter().map(Into::into).collect(),
            query: Vec::new(),
            form: Vec::new(),
            headers: HeaderMap::new(),
            body: None,
            timeout_override: None,
        }
    }

    #[must_use]
    pub fn get<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(Method::GET, segments)
    }

    #[must_use]
    pub fn post<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(Method::POST, segments)
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn put<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(Method::PUT, segments)
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn delete<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(Method::DELETE, segments)
    }

    #[must_use]
    pub fn query_pair(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
        self
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn query_pairs<I, K, V>(mut self, pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.query
            .extend(pairs.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn form_pair(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.body = None;
        self.form.push((key.into(), value.into()));
        self
    }

    #[must_use]
    pub fn form_pairs<I, K, V>(mut self, pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.body = None;
        self.form
            .extend(pairs.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(name, value);
        self
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    #[must_use]
    pub fn body(mut self, body: RequestBody) -> Self {
        self.form.clear();
        self.body = Some(body);
        self
    }

    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout_override = Some(timeout);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    #[cfg(feature = "unstable-raw")]
    pub retries: usize,
}

impl Response {
    #[must_use]
    #[cfg(feature = "unstable-raw")]
    pub fn text_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}
