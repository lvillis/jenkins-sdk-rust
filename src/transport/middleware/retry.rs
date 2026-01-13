use http::{HeaderMap, Method, StatusCode};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Retry configuration for both async and blocking clients.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries after the initial attempt.
    pub max_retries: usize,
    /// Base delay used for exponential backoff (`base * 2^n`).
    pub base_delay: Duration,
    /// Maximum delay cap for exponential backoff.
    pub max_delay: Duration,
    /// Add jitter to backoff delays to avoid retry storms.
    pub jitter: bool,
    /// Retry non-idempotent methods (e.g. `POST`). Defaults to `false`.
    pub retry_non_idempotent: bool,
    /// Prefer the server-provided `Retry-After` header when present.
    pub respect_retry_after: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            jitter: true,
            retry_non_idempotent: false,
            respect_retry_after: true,
        }
    }
}

impl RetryConfig {
    #[must_use]
    pub fn new(max_retries: usize, base_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            ..Self::default()
        }
    }
}

pub(crate) fn is_idempotent(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::PUT | Method::DELETE | Method::OPTIONS | Method::TRACE
    )
}

pub(crate) fn is_retryable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

pub(crate) fn backoff_delay(config: &RetryConfig, attempt: usize) -> Duration {
    if attempt == 0 {
        return Duration::ZERO;
    }

    let exp = 2u32.saturating_pow((attempt - 1).min(31) as u32);
    let scaled = config.base_delay.saturating_mul(exp);
    scaled.min(config.max_delay)
}

pub(crate) fn parse_retry_after(headers: &HeaderMap, now: SystemTime) -> Option<Duration> {
    let value = headers.get(http::header::RETRY_AFTER)?;
    let text = value.to_str().ok()?.trim();
    if text.is_empty() {
        return None;
    }

    if let Ok(secs) = text.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    let at = httpdate::parse_http_date(text).ok()?;
    let delay = at.duration_since(now).unwrap_or(Duration::ZERO);
    Some(delay)
}

pub(crate) fn jitter_delay(cap: Duration) -> Duration {
    if cap.is_zero() {
        return cap;
    }

    let max_ms = cap.as_millis().min(u128::from(u64::MAX)) as u64;
    if max_ms == 0 {
        return cap;
    }

    // Full jitter: random delay in [0, cap].
    let mut x = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos() as u64;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    let ms = x % (max_ms + 1);
    Duration::from_millis(ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, HeaderValue};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::RETRY_AFTER, HeaderValue::from_static("7"));
        let delay = parse_retry_after(&headers, UNIX_EPOCH).unwrap();
        assert_eq!(delay, Duration::from_secs(7));
    }

    #[test]
    fn retry_after_http_date() {
        let mut headers = HeaderMap::new();
        let now = UNIX_EPOCH + Duration::from_secs(100);
        let at = UNIX_EPOCH + Duration::from_secs(130);
        let value = httpdate::fmt_http_date(at);
        headers.insert(
            http::header::RETRY_AFTER,
            HeaderValue::from_str(&value).unwrap(),
        );
        let delay = parse_retry_after(&headers, now).unwrap();
        assert_eq!(delay, Duration::from_secs(30));
    }
}
