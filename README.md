<div align=right>Table of Contents↗️</div>

<h1 align=center><code>jenkins-sdk-rust</code></h1>

<p align=center>📦 Jenkins API SDK written in Rust</p>

<div align=center>
  <a href="https://crates.io/crates/jenkins-sdk">
    <img src="https://img.shields.io/crates/v/jenkins-sdk.svg" alt="crates.io version">
  </a>
  <a href="https://crates.io/crates/jenkins-sdk">
    <img src="https://img.shields.io/crates/dr/jenkins-sdk?color=ba86eb" alt="crates.io downloads">
  </a>
  <a href="https://github.com/lvillis/jenkins-sdk-rust">
    <img src="https://img.shields.io/github/repo-size/lvillis/jenkins-sdk-rust?style=flat-square&color=328657" alt="repo size">
  </a>
  <a href="https://github.com/lvillis/jenkins-sdk-rust/actions">
    <img src="https://github.com/lvillis/jenkins-sdk-rust/actions/workflows/ci.yaml/badge.svg" alt="build status">
  </a>
  <a href="mailto:lvillis@outlook.com?subject=Thanks%20for%20jenkins-sdk-rust!">
    <img src="https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg" alt="say thanks">
  </a>
</div>

---

`jenkins-sdk-rust` provides a modern, ergonomic interface for talking to Jenkins—from tiny CLI utilities to production
services.  
The crate ships **both asynchronous (Tokio) _and_ blocking** clients, exposes each REST endpoint as a strongly-typed
value, and offers a chainable builder with ready-made middleware such as **automatic CSRF-crumb fetching** and *
*exponential back-off retries**.

## ✨ Features

* **Async & Blocking** – pick the I/O model that fits your project at _compile time_. The default feature set builds the
  async client; enable `blocking-client` when you need synchronous calls.
* **Type-safe endpoints** – each API call is represented by a zero-cost struct implementing `Endpoint`; responses
  deserialize into concrete Rust types.
* **Composable middleware** – add automatic retries, CSRF crumbs, custom transports, or roll your own.
* **No magic strings** – URL construction, query/form encoding, error mapping, and JSON decoding are handled for you.
* **Pure Rust, small deps** – built on [`reqwest`](https://crates.io/crates/reqwest) with `rustls` TLS by default.

## 🚀 Supported API Endpoints

- **Job Management**
    - [x] Retrieve jobs information
    - [x] Fetch console logs
    - [x] Trigger builds with parameters
    - [x] Stop ongoing builds

- **Queue Management**
    - [x] Retrieve build queue details

- **Executor Management**
    - [x] Retrieve executor statistics and status

## 📥 Installation

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
jenkins-sdk = "0.1"
```

## ⚡Quick Start

### Async Example

```rust
use jenkins_sdk::{JenkinsAsync};
use jenkins_sdk::core::{QueueLength, JobsInfo, ExecutorsInfoEndpoint};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), jenkins_sdk::core::JenkinsError> {
    // Build a client with some sugar ‑‑>
    let j = JenkinsAsync::builder("https://jenkins.example.com")
        .auth_basic("user", "apitoken")
        .no_system_proxy()
        .with_retry(3, Duration::from_millis(300))
        .with_crumb(Duration::from_secs(1800))
        .build();

    // Queue length
    let q: serde_json::Value = j.request(&QueueLength).await?;
    println!("queue items = {}", q["items"].as_array().map_or(0, |a| a.len()));

    // Executor stats (typed deserialisation)
    let mut ex = j.request(&ExecutorsInfoEndpoint).await?;
    ex = ex.calc_idle();
    println!("idle executors = {}", ex.idle_executors);

    // Raw job list
    let jobs: serde_json::Value = j.request(&JobsInfo).await?;
    println!("first job = {}", jobs["jobs"][0]["name"]);

    Ok(())
}

```

### Sync Example

```rust
// Compile with `default-features = false, features = ["blocking-client"]`.
use jenkins_sdk::{JenkinsBlocking};
use jenkins_sdk::core::QueueLength;
use std::time::Duration;

fn main() -> Result<(), jenkins_sdk::core::JenkinsError> {
    let j = JenkinsBlocking::builder("https://jenkins.example.com")
        .auth_basic("user", "apitoken")
        .timeout(Duration::from_secs(15))
        .with_retry(2, Duration::from_millis(250))
        .build();

    let q: serde_json::Value = j.request(&QueueLength)?;
    println!("queue items = {}", q["items"].as_array().unwrap().len());
    Ok(())
}
```

## 📃 License

This project is licensed under the MIT License.
