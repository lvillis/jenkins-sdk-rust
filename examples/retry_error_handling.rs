//! Async retry configuration + error classification example.
//!
//! ```bash
//! cargo run --example retry_error_handling
//! ```
//!
//! Env vars:
//! - `JENKINS_URL`
//! - `JENKINS_USER`, `JENKINS_TOKEN` (optional)

use jenkins_sdk::{Client, Error, RetryConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = env_or("JENKINS_URL", "https://jenkins.example.com");

    let retry = RetryConfig {
        max_retries: 5,
        base_delay: Duration::from_millis(150),
        max_delay: Duration::from_secs(3),
        jitter: true,
        retry_non_idempotent: false,
        respect_retry_after: true,
    };

    let mut builder = Client::builder(&base_url)?
        .timeout(Duration::from_secs(15))
        .retry_config(retry);

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;

    match client.queue().list(Some("items[id]")).await {
        Ok(queue) => {
            let count = queue.items.len();
            println!("request succeeded, queue items={count}");
        }
        Err(err) => report_error(&err),
    }

    Ok(())
}

fn report_error(err: &Error) {
    eprintln!(
        "request failed: kind={:?}, retryable={}",
        err.kind(),
        err.is_retryable()
    );

    match err {
        Error::RateLimited { error, retry_after } => {
            eprintln!(
                "rate limited: status={}, path={}, retry_after={retry_after:?}",
                error.status,
                error.path()
            );
        }
        Error::Auth(http) | Error::NotFound(http) | Error::Conflict(http) | Error::Api(http) => {
            eprintln!(
                "http error: status={}, path={}, request_id={:?}",
                http.status,
                http.path(),
                http.request_id
            );
            if let Some(snippet) = http.body_snippet.as_deref() {
                eprintln!("body_snippet={snippet}");
            }
        }
        Error::Transport {
            kind, method, path, ..
        } => {
            eprintln!("transport error: kind={kind:?}, method={method}, path={path}");
        }
        Error::Decode {
            status,
            method,
            path,
            ..
        } => {
            eprintln!("decode error: status={status}, method={method}, path={path}");
        }
        Error::InvalidConfig { message, .. } => {
            eprintln!("invalid config: {message}");
        }
        _ => {
            eprintln!("unclassified error variant: {err}");
        }
    }
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
