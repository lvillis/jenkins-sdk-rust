//! Async TLS root-store policy example.
//!
//! ```bash
//! cargo run --example tls_root_store
//! ```
//!
//! Env vars:
//! - `JENKINS_URL`
//! - `JENKINS_USER`, `JENKINS_TOKEN` (optional)
//! - `JENKINS_TLS_ROOT_STORE` = `backend-default` | `system` | `webpki`
//! - `JENKINS_NO_SYSTEM_PROXY` = `1|true|yes|on` (optional)

use jenkins_sdk::{Client, TlsRootStore};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = env_or("JENKINS_URL", "https://jenkins.example.com");
    let tls_root_store = parse_tls_root_store(env_or("JENKINS_TLS_ROOT_STORE", "backend-default"));

    let mut builder = Client::builder(&base_url)?
        .timeout(Duration::from_secs(20))
        .tls_root_store(tls_root_store);

    if env_bool("JENKINS_NO_SYSTEM_PROXY") {
        builder = builder.no_system_proxy();
    }

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;
    let queue = client.queue().list(Some("items[id]")).await?;
    let items = queue.items.len();
    println!("tls_root_store={tls_root_store:?}, queue items={items}");

    Ok(())
}

fn parse_tls_root_store(value: String) -> TlsRootStore {
    match value.trim().to_ascii_lowercase().as_str() {
        "backend-default" | "default" => TlsRootStore::BackendDefault,
        "system" => TlsRootStore::System,
        "webpki" | "web-pki" => TlsRootStore::WebPki,
        _ => TlsRootStore::BackendDefault,
    }
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn env_bool(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}
