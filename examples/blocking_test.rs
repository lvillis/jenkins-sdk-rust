//! End-to-end blocking demo using builder sugar.
//!
//! Build with:
//! ```bash
//! cargo run --no-default-features --features blocking-rustls-ring --example blocking_test
//! # or: cargo run --no-default-features --features blocking-rustls-aws-lc-rs --example blocking_test
//! # or: cargo run --no-default-features --features blocking-native-tls --example blocking_test
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
        .timeout(Duration::from_secs(20))
        .with_retry(2, Duration::from_millis(200))
        .with_crumb(Duration::from_secs(1_800));

    if let (Some(user), Some(token)) = (env_opt("JENKINS_USER"), env_opt("JENKINS_TOKEN")) {
        builder = builder.auth_basic(user, token);
    }

    let client = builder.build()?;

    // who am I
    let me = client.system().who_am_i()?;
    println!(
        "whoAmI: {}",
        me.full_name.as_deref().unwrap_or(
            me.name
                .as_deref()
                .or(me.id.as_deref())
                .unwrap_or("<unknown>"),
        )
    );

    // queue length (cheap tree)
    let queue = client.queue().list(Some("items[id]"))?;
    let items = queue.items.len();
    println!("queue items: {items}");

    // executors (typed)
    let exec = client.computers().executors_info()?.calc_idle();
    println!(
        "executors: total={}, busy={}, idle={}",
        exec.total_executors, exec.busy_executors, exec.idle_executors
    );

    // job list
    let jobs = client.jobs().list()?;
    if !jobs.jobs.is_empty() {
        println!("first three jobs:");
        for job in jobs.jobs.iter().take(3) {
            println!("  - {}", job.name);
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
