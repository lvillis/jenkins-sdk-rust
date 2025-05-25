//! Shared error enum.

use http::StatusCode;
use thiserror::Error;
use url::ParseError;

/// All errors returned by the SDK.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JenkinsError {
    /// Non-2xx HTTP response.
    #[error("HTTP {code}: {body}")]
    Http { code: StatusCode, body: String },

    /// Transport failure (`reqwest`).
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// URL construction failure.
    #[error(transparent)]
    Url(#[from] ParseError),

    /// JSON deserialization failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
