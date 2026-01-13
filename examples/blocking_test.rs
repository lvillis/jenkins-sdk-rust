//! End-to-end blocking demo using builder sugar.
//!
//! Build with:
//! ```bash
//! cargo run --no-default-features --features blocking,rustls --example blocking_test
//! # or: cargo run --no-default-features --features blocking,native-tls --example blocking_test
//! ```
//!
//! Set env vars to run against a real Jenkins:
//! - `JENKINS_URL` (e.g. `https://jenkins.example.com`)
//! - `JENKINS_USER`, `JENKINS_TOKEN` (optional, but most instances require auth)
//! - `JENKINS_JOB` (default: `core`)
//! - `JENKINS_BUILD` (default: `lastBuild`)
//! - `JENKINS_TRIGGER=1` to actually trigger a build (POST)

use jenkins_sdk::BlockingClient;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let base_url = env_or("JENKINS_URL", "https://jenkins.example.com");
    let job = env_or("JENKINS_JOB", "core");
    let build = env_or("JENKINS_BUILD", "lastBuild");
    let trigger = env_bool("JENKINS_TRIGGER");

    let mut builder = BlockingClient::builder(&base_url)?
        .no_system_proxy()
        .timeout(Duration::from_secs(20))
        .with_retry(2, Duration::from_millis(200))
        .with_crumb(Duration::from_secs(1_800));

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;

    // who am I
    let me: serde_json::Value = client.system().who_am_i()?;
    println!(
        "whoAmI: {}",
        me.get("fullName")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>")
    );

    // queue length (cheap tree)
    let queue: serde_json::Value = client.queue().list(Some("items[id]"))?;
    let items = queue
        .get("items")
        .and_then(|v| v.as_array())
        .map_or(0, |a| a.len());
    println!("queue items: {items}");

    // executors (typed)
    let exec = client.computers().executors_info()?.calc_idle();
    println!(
        "executors: total={}, busy={}, idle={}",
        exec.total_executors, exec.busy_executors, exec.idle_executors
    );

    // job list
    let jobs: serde_json::Value = client.jobs().list()?;
    if let Some(arr) = jobs.get("jobs").and_then(|v| v.as_array()) {
        println!("first three jobs:");
        for job in arr.iter().take(3) {
            let name = job
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            println!("  - {name}");
        }
    }

    // console log snippet (build selector like `lastBuild` is accepted by Jenkins)
    let chunk = client
        .jobs()
        .progressive_console_text(job.as_str(), build.as_str(), 0)?;
    println!("log chunk (start=0):\n{}", tail_chars(&chunk.text, 200));

    if trigger {
        let triggered = client
            .jobs()
            .build_with_parameters(job.as_str(), [("foo", "1"), ("env", "dev")])?;
        println!("triggered: {triggered:?}");
    } else {
        println!("skipping trigger (set JENKINS_TRIGGER=1 to enable)");
    }

    Ok(())
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

fn tail_chars(s: &str, max_chars: usize) -> &str {
    if max_chars == 0 {
        return "";
    }

    let mut iter = s.char_indices().rev();
    let mut start = None;
    for _ in 0..max_chars {
        match iter.next() {
            Some((idx, _)) => start = Some(idx),
            None => return s,
        }
    }
    &s[start.unwrap_or(0)..]
}
