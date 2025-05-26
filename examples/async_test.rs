//! End-to-end async demo:
//! 1. queue length   2. executors info   3. job list
//! 4. console text   5. trigger build

use jenkins_sdk::{
    JenkinsAsync, StopBuild,
    core::{
        ConsoleText, ExecutorsInfo, ExecutorsInfoEndpoint, JobsInfo, QueueLength, TriggerBuild,
    },
};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) build async client
    let client = JenkinsAsync::builder("https://jenkins.example.com")
        .auth_basic("user", "apitoken")
        .no_system_proxy()
        .with_retry(3, Duration::from_millis(200))
        .with_crumb(Duration::from_secs(1800))
        .build();

    // 2) queue length
    let queue: serde_json::Value = client.request(&QueueLength).await?;
    let items = queue["items"].as_array().map_or(0, |a| a.len());
    println!("Queue length: {items}");

    // 3) executors
    let mut exec: ExecutorsInfo = client.request(&ExecutorsInfoEndpoint).await?;
    exec = exec.calc_idle();
    println!(
        "Executors -> total: {}, busy: {}, idle: {}",
        exec.total_executors, exec.busy_executors, exec.idle_executors
    );

    // 4) job list
    let jobs: serde_json::Value = client.request(&JobsInfo).await?;
    println!("First three jobs:");
    for j in jobs["jobs"].as_array().unwrap().iter().take(3) {
        println!("  • {}", j["name"]);
    }

    // 5) console text of build #42
    let log: String = client.request(&ConsoleText("core", "91")).await?;
    println!("Last 120 chars:\n{}", &log[log.len().saturating_sub(120)..]);

    // 6) trigger new build with parameters
    let params = json!({ "foo": "1", "env": "dev" });
    client
        .request(&TriggerBuild {
            job: "core",
            params: &params,
        })
        .await?;
    println!("Triggered build with params {params}");

    // Stop build #91 in the "core" job
    let resp: String = client
        .request(&StopBuild {
            job: "core",
            build: "91",
        })
        .await?;
    println!("StopBuild response: {resp}");

    Ok(())
}
