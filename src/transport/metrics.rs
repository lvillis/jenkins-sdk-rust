use crate::ErrorKind;
use http::{Method, StatusCode};
use std::time::Duration;

pub(crate) struct InFlightGuard {
    gauge: metrics::Gauge,
}

impl InFlightGuard {
    pub(crate) fn new() -> Self {
        let gauge = metrics::gauge!("jenkins_sdk_inflight");
        gauge.increment(1.0);
        Self { gauge }
    }
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.gauge.decrement(1.0);
    }
}

fn status_class(status: StatusCode) -> &'static str {
    match status.as_u16() {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}

fn error_kind_label(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Auth => "auth",
        ErrorKind::NotFound => "not_found",
        ErrorKind::Conflict => "conflict",
        ErrorKind::RateLimited => "rate_limited",
        ErrorKind::Api => "api",
        ErrorKind::Transport => "transport",
        ErrorKind::Decode => "decode",
        ErrorKind::InvalidConfig => "invalid_config",
    }
}

fn method_label(method: &Method) -> metrics::SharedString {
    match method {
        &Method::GET => "GET".into(),
        &Method::POST => "POST".into(),
        &Method::PUT => "PUT".into(),
        &Method::DELETE => "DELETE".into(),
        &Method::PATCH => "PATCH".into(),
        &Method::HEAD => "HEAD".into(),
        &Method::OPTIONS => "OPTIONS".into(),
        &Method::CONNECT => "CONNECT".into(),
        &Method::TRACE => "TRACE".into(),
        other => other.to_string().into(),
    }
}

pub(crate) fn record_outcome(
    method: &Method,
    status: Option<StatusCode>,
    latency: Duration,
    retries: usize,
    error_kind: Option<ErrorKind>,
) {
    let method = method_label(method);
    let status_class = status.map(status_class).unwrap_or("transport");

    metrics::counter!(
        "jenkins_sdk_requests_total",
        "method" => method.clone(),
        "status_class" => status_class
    )
    .increment(1);
    metrics::histogram!(
        "jenkins_sdk_request_duration_seconds",
        "method" => method.clone(),
        "status_class" => status_class
    )
    .record(latency);

    if retries > 0 {
        metrics::counter!("jenkins_sdk_retries_total", "method" => method.clone())
            .increment(retries as u64);
    }

    if status == Some(StatusCode::TOO_MANY_REQUESTS) {
        metrics::counter!("jenkins_sdk_rate_limited_total", "method" => method.clone())
            .increment(1);
    }

    if let Some(kind) = error_kind {
        metrics::counter!(
            "jenkins_sdk_errors_total",
            "method" => method,
            "kind" => error_kind_label(kind)
        )
        .increment(1);
    }
}
