//! Minimal async auth + whoAmI example.
//!
//! ```bash
//! cargo run --example whoami
//! ```
//!
//! Env vars:
//! - `JENKINS_URL`
//! - `JENKINS_USER`, `JENKINS_TOKEN` (optional)

use jenkins_sdk::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = env_or("JENKINS_URL", "https://jenkins.example.com");

    let mut builder = Client::builder(&base_url)?
        .no_system_proxy()
        .timeout(Duration::from_secs(30));

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;
    let me: serde_json::Value = client.system().who_am_i().await?;
    println!("{me:#}");
    Ok(())
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
