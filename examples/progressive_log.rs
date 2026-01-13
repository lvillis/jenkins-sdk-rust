//! Async progressive console log example.
//!
//! ```bash
//! cargo run --example progressive_log
//! ```
//!
//! Env vars:
//! - `JENKINS_URL`
//! - `JENKINS_USER`, `JENKINS_TOKEN` (optional)
//! - `JENKINS_JOB` (default: `core`)
//! - `JENKINS_BUILD` (default: `lastBuild`)

use jenkins_sdk::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = env_or("JENKINS_URL", "https://jenkins.example.com");
    let job = env_or("JENKINS_JOB", "core");
    let build = env_or("JENKINS_BUILD", "lastBuild");

    let mut builder = Client::builder(&base_url)?
        .no_system_proxy()
        .timeout(Duration::from_secs(30));

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;

    let mut start = 0u64;
    for _ in 0..10 {
        let chunk = client
            .jobs()
            .progressive_console_text(job.as_str(), build.as_str(), start)
            .await?;

        if !chunk.text.is_empty() {
            print!("{}", chunk.text);
        }

        if !chunk.more_data {
            break;
        }

        start = match chunk.next_start {
            Some(next) => next,
            None => break,
        };
    }

    Ok(())
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
