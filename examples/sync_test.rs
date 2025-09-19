//! End-to-end blocking demo using builder sugar.
//!
//! Build with:
//! ```bash
//! cargo run --no-default-features --features blocking-client --example sync_test
//! ```

#![cfg(feature = "blocking-client")]

use jenkins_sdk::{
    JenkinsBlocking, StopBuild,
    core::{
        ConsoleText, ExecutorsInfo, ExecutorsInfoEndpoint, JobsInfo, QueueLength, TriggerBuild,
    },
};
use serde_json::json;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // 1) build blocking client
    let client = JenkinsBlocking::builder("https://jenkins.example.com")
        .auth_basic("user", "apitoken")
        .no_system_proxy()
        .timeout(Duration::from_secs(20))
        .with_retry(2, Duration::from_millis(200))
        .with_crumb(Duration::from_secs(1_800))?
        .build()?;

    // 2) queue length
    let queue: serde_json::Value = client.request(&QueueLength)?;
    let items = queue["items"].as_array().map_or(0, |a| a.len());
    println!("Queue length: {items}");

    // 3) executors
    let mut exec: ExecutorsInfo = client.request(&ExecutorsInfoEndpoint)?;
    exec = exec.calc_idle();
    println!(
        "Executors -> total: {}, busy: {}, idle: {}",
        exec.total_executors, exec.busy_executors, exec.idle_executors
    );

    // 4) job list
    let jobs: serde_json::Value = client.request(&JobsInfo)?;
    println!("First three jobs:");
    for job in jobs["jobs"].as_array().unwrap().iter().take(3) {
        println!("  - {}", job["name"]);
    }

    // 5) console text of build #91
    let log: String = client.request(&ConsoleText("core", "91"))?;
    println!("Last 120 chars:\n{}", &log[log.len().saturating_sub(120)..]);

    // 6) trigger new build with parameters
    let params = json!({ "foo": "1", "env": "dev" });
    client.request(&TriggerBuild {
        job: "core",
        params: &params,
    })?;
    println!("Triggered build with params {params}");

    // Stop build #91 in the "core" job
    let resp: String = client.request(&StopBuild {
        job: "core",
        build: "91",
    })?;
    println!("StopBuild response: {resp}");

    Ok(())
}

/// Shuts up IDE diagnostics when the `blocking-client` feature is **OFF**.
#[cfg(not(feature = "blocking-client"))]
fn main() {
    // Intentionally empty: build the example with
    // `cargo run --no-default-features --features blocking-client --example sync_test`
}
