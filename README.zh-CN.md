<!-- ─── 语言切换 & 目录（右上角） ─────────────────────────────── -->
<div align="right">

<a href="README.md">🇺🇸 English</a> ·
<a aria-disabled="true" style="color:#999;text-decoration:none;">🇨🇳 中文</a> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 目录 ↗️

</div>

<h1 align="center"><code>jenkins-sdk-rust</code></h1>

<p align="center">
  📦 <strong>纯 Rust 实现的 Jenkins API SDK</strong> — 同时支持 <em>异步</em> 与 <em>阻塞</em> 客户端、类型安全端点、可插拔中间件，拒绝魔法字符串。
</p>

<div align="center">
  <a href="https://crates.io/crates/jenkins-sdk">
    <img src="https://img.shields.io/crates/v/jenkins-sdk.svg" alt="Crates.io 版本">
  </a>
  <a href="https://docs.rs/jenkins-sdk">
    <img src="https://img.shields.io/docsrs/jenkins-sdk?logo=rust" alt="文档">
  </a>
  <a href="https://github.com/lvillis/jenkins-sdk-rust/actions">
    <img src="https://github.com/lvillis/jenkins-sdk-rust/actions/workflows/ci.yaml/badge.svg" alt="CI 状态">
  </a>
  <a href="https://img.shields.io/crates/dr/jenkins-sdk?color=ba86eb">
    <img src="https://img.shields.io/crates/dr/jenkins-sdk?color=ba86eb" alt="下载量">
  </a>
  <a href="https://github.com/lvillis/jenkins-sdk-rust">
    <img src="https://img.shields.io/github/repo-size/lvillis/jenkins-sdk-rust?color=328657&style=flat-square" alt="仓库大小">
  </a>
  <a href="mailto:lvillis@outlook.com?subject=Thanks%20for%20jenkins-sdk-rust!">
    <img src="https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg" alt="say thanks">
  </a>
</div>

---

## ✨ 功能特性

| 功能             | 说明                                                |
|----------------|---------------------------------------------------|
| **异步 & 阻塞**    | 编译期选择 I/O 模型：默认 `tokio`，如需同步启用 `blocking`。 |
| **核心 Service** | 通过 `client.jobs()/queue()/system()/...` 访问 Jenkins Core API，无需手写路径。 |
| **可组合中间件**     | 内置 CSRF Crumb 获取、重试、定制传输等，链式调用自由扩展。               |
| **无魔法字符串**     | URL 构造、查询/表单编码、错误映射、JSON 解码全由 SDK 处理。             |
| **默认纯 Rust** | 基于 `reqx`，默认 TLS 为 `rustls`，并可按需切换 `native-tls`。 |

## 🖼 架构总览

<details open>
<summary>点击折叠 / 展开</summary>

```mermaid
flowchart LR
%% —— 你的应用 ——
    subgraph A["你的应用"]
        direction TB
        CLI["二进制 / 服务"]
    end

%% —— SDK 核心 ——
    subgraph S["jenkins-sdk-rust"]
        direction LR
        Builder["客户端构建器"] --> Client["Jenkins<br/>异步 / 阻塞"] --> Middleware["中间件<br/><sub>重试 • 取 Crumb • 自定义</sub>"] --> Service["核心 Service<br/><sub>jobs • queue • system • ...</sub>"]
    end

%% —— Jenkins 主机 ——
    subgraph J["Jenkins 主机"]
        API["REST API"]
    end

%% —— 流向 ——
    CLI --> Builder
    Service --> API

%% —— 样式 ——
    classDef app fill:#e3f2fd,stroke:#1976d2,stroke-width:1px;
    classDef sdk fill:#e8f5e9,stroke:#388e3c,stroke-width:1px;
    classDef server fill:#fff8e1,stroke:#f57f17,stroke-width:1px;

    class CLI app;
    class Builder,Client,Middleware,Service sdk;
    class API server;
```

</details>

## 🚀 支持的 API 列表

| Service         | Jenkins Core API                                                                                          | 状态 |
|----------------|-----------------------------------------------------------------------------------------------------------|----|
| `system()`     | 类型化 root/whoAmI/crumb + `/overallLoad/api/json` 与 `/loadStatistics/api/json`、`/jnlpJars/agent.jar`、`/jnlpJars/jenkins-cli.jar`、`/config.xml` get/update、`/quietDown`/`cancelQuietDown`/`reload`/`safeRestart`/`restart`/`exit` | ✅  |
| `jobs()`       | 类型化 list/get + 类型化 lastBuild selectors/build info、`consoleText`、`logText/progressiveText`、artifact download、stop/term/kill/doDelete/toggleLogKeep/submitDescription、`config.xml` get/update、`createItem`(xml)、copy/rename/delete/enable/disable | ✅  |
| `queue()`      | 类型化 list/item + cancel                                                                                   | ✅  |
| `computers()`  | 类型化 list/computer + 类型化 `executors_info()`、`doCreateItem`(xml)/copy、toggleOffline/doDelete、`config.xml` get/update、connect/disconnect/launchSlaveAgent | ✅  |
| `views()`      | 类型化 list/get、createView(xml)、`config.xml` get/update、doDelete/doRename、addJobToView/removeJobFromView | ✅  |
| `users()`      | 类型化 `/user/<id>/api/json`、类型化 `/whoAmI/api/json`、`config.xml` get/update | ✅  |
| `people()`     | 类型化 `/people/api/json`、类型化 `/asynchPeople/api/json` | ✅  |

## 📥 安装

```shell
# 一键添加依赖（异步客户端）
cargo add jenkins-sdk
```

```toml
# Cargo.toml（默认异步客户端）
[dependencies]
jenkins-sdk = "0.1"

# 异步客户端（显式选择 TLS 后端）
# jenkins-sdk = { version = "0.1", default-features = false, features = ["async-rustls-ring"] }
# jenkins-sdk = { version = "0.1", default-features = false, features = ["async-rustls-aws-lc-rs"] }
# jenkins-sdk = { version = "0.1", default-features = false, features = ["async-native-tls"] }

# 同步客户端（TLS 二选一）
# jenkins-sdk = { version = "0.1", default-features = false, features = ["blocking-rustls-ring"] }
# jenkins-sdk = { version = "0.1", default-features = false, features = ["blocking-rustls-aws-lc-rs"] }
# jenkins-sdk = { version = "0.1", default-features = false, features = ["blocking-native-tls"] }
```

## ⚡快速开始

> Jenkins 基础地址可以带子路径（如 `https://example.com/jenkins`），尾部 `/` 可省略，SDK 会自动补齐并正常拼接 API 路径。

### 异步示例

```rust
use jenkins_sdk::{Client, TlsRootStore};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 构建客户端
    let jenkins = Client::builder("https://jenkins.example.com")?
        .auth_basic("user", "apitoken")
        // 可选：显式指定 TLS 根证书策略
        .tls_root_store(TlsRootStore::BackendDefault)
        .with_retry(3, Duration::from_millis(300))
        .with_crumb(Duration::from_secs(1800))
        .build()?;

    // 队列长度
    let q = jenkins.queue().list(None).await?;
    println!("队列条目数 = {}", q.items.len());

    // 执行器状态
    let ex = jenkins.computers().executors_info().await?;
    println!("空闲执行器 = {}", ex.idle_executors);

    Ok(())
}
```

默认会读取系统代理环境变量（`HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY`）。
如需禁用该行为，可调用 `.no_system_proxy()`。

### 阻塞示例

```rust
// 需要以下三选一:
// default-features = false, features = ["blocking-rustls-ring"]
// default-features = false, features = ["blocking-rustls-aws-lc-rs"]
// default-features = false, features = ["blocking-native-tls"]
use jenkins_sdk::BlockingClient;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let jenkins = BlockingClient::builder("https://jenkins.example.com")?
        .auth_basic("user", "apitoken")
        .timeout(Duration::from_secs(15))
        .with_retry(2, Duration::from_millis(250))
        .build()?;

    let q = jenkins.queue().list(None)?;
    println!("队列条目数 = {}", q.items.len());

    Ok(())
}
```

> 注意：在 Tokio runtime 中使用阻塞客户端时，请通过 `tokio::task::spawn_blocking` 或专用线程池调用。

## 📜 更新日志

详细变更请见 [CHANGELOG.md](CHANGELOG.md) .

## 📃 许可证

本项目基于 MIT License 发布
