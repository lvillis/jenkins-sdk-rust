//! Async artifact download example.
//!
//! ```bash
//! cargo run --example artifact_download
//! ```
//!
//! Env vars:
//! - `JENKINS_URL`
//! - `JENKINS_USER`, `JENKINS_TOKEN` (optional)
//! - `JENKINS_JOB` (default: `core`)
//! - `JENKINS_BUILD` (default: `lastSuccessfulBuild`)
//! - `JENKINS_ARTIFACT` (required, e.g. `target/app.tar.gz` or `a/b c.txt`)
//! - `OUTPUT_PATH` (optional, write the downloaded bytes to this path)

use jenkins_sdk::Client;
use std::{path::PathBuf, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = env_or("JENKINS_URL", "https://jenkins.example.com");
    let job = env_or("JENKINS_JOB", "core");
    let build = env_or("JENKINS_BUILD", "lastSuccessfulBuild");

    let Some(artifact) = env_opt("JENKINS_ARTIFACT") else {
        eprintln!("missing JENKINS_ARTIFACT (example: target/app.tar.gz)");
        return Ok(());
    };

    let mut builder = Client::builder(&base_url)?
        .no_system_proxy()
        .timeout(Duration::from_secs(60));

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;

    let bytes = client
        .jobs()
        .download_artifact(job.as_str(), build.as_str(), artifact)
        .await?;

    if let Some(output) = env_opt("OUTPUT_PATH") {
        let path = PathBuf::from(output);
        std::fs::write(&path, &bytes)?;
        println!("wrote {} bytes to {}", bytes.len(), path.display());
    } else {
        println!("downloaded {} bytes", bytes.len());
    }

    Ok(())
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
