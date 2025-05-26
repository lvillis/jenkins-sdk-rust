use http::{Method, StatusCode};
use thiserror::Error;
use url::{ParseError, Url};

/// All errors returned by the SDK.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JenkinsError {
    /// Non-2xx HTTP response.
    #[error("HTTP {code} ({method} {url}): {body}")]
    Http {
        code: StatusCode,
        method: Method,
        url: Url,
        body: String,
    },

    /// Transport failure (`reqwest`).
    #[error("Transport error during {method} {url}: {source}")]
    Reqwest {
        #[source]
        source: reqwest::Error,
        method: Method,
        url: Url,
    },

    /// URL construction failure.
    #[error(transparent)]
    Url(#[from] ParseError),

    /// JSON deserialization failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
