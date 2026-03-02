//! Async request hook example.
//!
//! ```bash
//! cargo run --example request_hook
//! ```
//!
//! Env vars:
//! - `JENKINS_URL`
//! - `JENKINS_USER`, `JENKINS_TOKEN` (optional)

use jenkins_sdk::Client;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = env_or("JENKINS_URL", "https://jenkins.example.com");
    let hook_calls = Arc::new(AtomicUsize::new(0));
    let hook_calls_for_closure = Arc::clone(&hook_calls);

    let mut builder = Client::builder(&base_url)?
        .timeout(Duration::from_secs(20))
        .request_hook(move |ctx| {
            let call_no = hook_calls_for_closure.fetch_add(1, Ordering::Relaxed) + 1;
            println!(
                "[hook #{call_no}] {} {} query={} form={} body={}",
                ctx.method,
                ctx.url.path(),
                ctx.query.len(),
                ctx.form.len(),
                ctx.body.map_or(0, |b| b.len())
            );
            // Demonstrate header mutation before request dispatch.
            ctx.headers.insert(
                http::header::HeaderName::from_static("x-sdk-example"),
                http::HeaderValue::from_static("request-hook"),
            );
            Ok(())
        });

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;

    // A couple of requests so hook output is visible.
    let _whoami = client.system().who_am_i().await?;
    let _queue = client.queue().list(Some("items[id]")).await?;

    println!(
        "hook was called {} times",
        hook_calls.load(Ordering::Relaxed)
    );
    Ok(())
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
