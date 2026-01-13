use http::{Method, StatusCode};
use std::{error::Error as StdError, fmt, time::Duration};
use thiserror::Error;
use url::Url;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub struct BodySnippetConfig {
    pub enabled: bool,
    pub max_bytes: usize,
}

impl Default for BodySnippetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_bytes: 4096,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    Auth,
    NotFound,
    Conflict,
    RateLimited,
    Api,
    Transport,
    Decode,
    InvalidConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportErrorKind {
    Timeout,
    Connect,
    Other,
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub status: StatusCode,
    pub method: Method,
    /// Sanitized URL: no query/fragment/userinfo.
    pub url: Box<Url>,
    pub message: Option<Box<str>>,
    pub request_id: Option<Box<str>>,
    pub body_snippet: Option<Box<str>>,
}

impl HttpError {
    #[must_use]
    pub fn path(&self) -> &str {
        self.url.path()
    }
}

/// All errors returned by the SDK.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    Auth(HttpError),

    #[error("{0}")]
    NotFound(HttpError),

    #[error("{0}")]
    Conflict(HttpError),

    #[error("{error}")]
    RateLimited {
        error: HttpError,
        retry_after: Option<Duration>,
    },

    #[error("{0}")]
    Api(HttpError),

    #[error("Transport error during {method} {path}: {source}")]
    Transport {
        method: Method,
        path: Box<str>,
        kind: TransportErrorKind,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("Decode error (HTTP {status}) during {method} {path}: {source}")]
    Decode {
        status: StatusCode,
        method: Method,
        path: Box<str>,
        request_id: Option<Box<str>>,
        body_snippet: Option<Box<str>>,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        message: Box<str>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

impl Error {
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Auth(_) => ErrorKind::Auth,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::Conflict(_) => ErrorKind::Conflict,
            Self::RateLimited { .. } => ErrorKind::RateLimited,
            Self::Api(_) => ErrorKind::Api,
            Self::Transport { .. } => ErrorKind::Transport,
            Self::Decode { .. } => ErrorKind::Decode,
            Self::InvalidConfig { .. } => ErrorKind::InvalidConfig,
        }
    }

    #[must_use]
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::Auth(e) | Self::NotFound(e) | Self::Conflict(e) | Self::Api(e) => Some(e.status),
            Self::RateLimited { error, .. } => Some(error.status),
            Self::Decode { status, .. } => Some(*status),
            Self::Transport { .. } | Self::InvalidConfig { .. } => None,
        }
    }

    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Auth(e) | Self::NotFound(e) | Self::Conflict(e) | Self::Api(e) => {
                e.request_id.as_deref()
            }
            Self::RateLimited { error, .. } => error.request_id.as_deref(),
            Self::Decode { request_id, .. } => request_id.as_deref(),
            Self::Transport { .. } | Self::InvalidConfig { .. } => None,
        }
    }

    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    #[must_use]
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth(_))
    }

    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimited { .. } => true,
            Self::Api(e) => matches!(
                e.status,
                StatusCode::BAD_GATEWAY
                    | StatusCode::SERVICE_UNAVAILABLE
                    | StatusCode::GATEWAY_TIMEOUT
            ),
            Self::Transport { kind, .. } => matches!(
                kind,
                TransportErrorKind::Timeout | TransportErrorKind::Connect
            ),
            _ => false,
        }
    }

    pub(crate) fn from_http(error: HttpError, retry_after: Option<Duration>) -> Self {
        match error.status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Self::Auth(error),
            StatusCode::NOT_FOUND => Self::NotFound(error),
            StatusCode::CONFLICT | StatusCode::PRECONDITION_FAILED => Self::Conflict(error),
            StatusCode::TOO_MANY_REQUESTS => Self::RateLimited { error, retry_after },
            _ => Self::Api(error),
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP {} ({} {})", self.status, self.method, self.path())?;
        if let Some(message) = self.message.as_deref() {
            write!(f, ": {message}")?;
        }
        if let Some(request_id) = self.request_id.as_deref() {
            write!(f, " [request-id: {request_id}]")?;
        }
        Ok(())
    }
}
