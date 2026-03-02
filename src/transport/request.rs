use http::{HeaderMap, HeaderValue, Method, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct RequestBody {
    pub bytes: Vec<u8>,
    pub content_type: Option<HeaderValue>,
}

impl RequestBody {
    #[must_use]
    pub fn bytes_with_content_type(bytes: Vec<u8>, content_type: HeaderValue) -> Self {
        Self {
            bytes,
            content_type: Some(content_type),
        }
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
    pub fn query_pair(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
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
    pub fn body(mut self, body: RequestBody) -> Self {
        self.form.clear();
        self.body = Some(body);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Response {
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}
