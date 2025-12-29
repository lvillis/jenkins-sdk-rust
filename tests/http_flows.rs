use std::time::Duration;

use anyhow::Result;
use http::StatusCode;
#[cfg(feature = "blocking-client")]
use jenkins_sdk::JenkinsBlocking;
#[cfg(feature = "blocking-client")]
use jenkins_sdk::core::StopBuild;
use jenkins_sdk::{
    JenkinsAsync,
    core::{JenkinsError, JobsInfo, QueueLength, TriggerBuild},
};
use serde_json::json;
#[cfg(feature = "blocking-client")]
use tokio::task;
use tokio::time::sleep;
use wiremock::{
    Match, Mock, MockServer, Request, ResponseTemplate,
    matchers::{body_string_contains, header, method, path, query_param},
};

#[derive(Clone, Copy)]
struct CrumbHeader(&'static str);

impl Match for CrumbHeader {
    fn matches(&self, request: &Request) -> bool {
        request
            .headers
            .get("Jenkins-Crumb")
            .and_then(|value| value.to_str().ok())
            .map(|value| value == self.0)
            .unwrap_or(false)
    }
}

async fn mock_crumb(server: &MockServer, crumb: &'static str, expected: u64) {
    Mock::given(method("GET"))
        .and(path("/crumbIssuer/api/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "crumbRequestField": "Jenkins-Crumb",
            "crumb": crumb
        })))
        .expect(expected)
        .up_to_n_times(expected)
        .mount(server)
        .await;
}

async fn mock_post_with_auth(
    server: &MockServer,
    endpoint: &str,
    crumb: Option<&'static str>,
    body_snippet: Option<&'static str>,
    response: ResponseTemplate,
    expected: u64,
) {
    let mut mock = Mock::given(method("POST"))
        .and(path(endpoint))
        .and(header("Authorization", "Basic dXNlcjp0b2tlbg=="));

    if let Some(token) = crumb {
        mock = mock.and(CrumbHeader(token));
    }

    if let Some(snippet) = body_snippet {
        mock = mock.and(body_string_contains(snippet));
    }

    mock.respond_with(response)
        .expect(expected)
        .up_to_n_times(expected)
        .mount(server)
        .await;
}

async fn mock_get(server: &MockServer, endpoint: &str, response: ResponseTemplate, expected: u64) {
    Mock::given(method("GET"))
        .and(path(endpoint))
        .respond_with(response)
        .expect(expected)
        .up_to_n_times(expected)
        .mount(server)
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_client_refreshes_expired_crumb() -> Result<()> {
    let server = MockServer::start().await;

    mock_crumb(&server, "token-1", 1).await;
    mock_crumb(&server, "token-2", 1).await;

    mock_post_with_auth(
        &server,
        "/job/demo/buildWithParameters",
        Some("token-1"),
        None,
        ResponseTemplate::new(200).set_body_string("one"),
        1,
    )
    .await;

    mock_post_with_auth(
        &server,
        "/job/demo/buildWithParameters",
        Some("token-2"),
        None,
        ResponseTemplate::new(200).set_body_string("two"),
        1,
    )
    .await;

    let client = JenkinsAsync::builder(server.uri())
        .auth_basic("user", "token")
        .with_crumb(Duration::from_millis(0))?
        .build()?;

    let first = client
        .request(&TriggerBuild {
            job: "demo",
            params: &json!({ "foo": "bar" }),
        })
        .await?;
    assert_eq!(first, "one");

    sleep(Duration::from_millis(5)).await;

    let second = client
        .request(&TriggerBuild {
            job: "demo",
            params: &json!({ "foo": "baz" }),
        })
        .await?;
    assert_eq!(second, "two");

    server.verify().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_client_propagates_http_error() -> Result<()> {
    let server = MockServer::start().await;

    mock_get(
        &server,
        "/queue/api/json",
        ResponseTemplate::new(404).set_body_json(json!({
            "message": "queue not found"
        })),
        1,
    )
    .await;

    let client = JenkinsAsync::builder(server.uri()).build()?;

    let err = client
        .request(&QueueLength)
        .await
        .expect_err("expected HTTP error");

    match err {
        JenkinsError::Http { code, body, .. } => {
            assert_eq!(code, StatusCode::NOT_FOUND);
            assert!(body.contains("queue not found"));
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.verify().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_client_attaches_crumb_and_basic_auth() -> Result<()> {
    let server = MockServer::start().await;

    mock_crumb(&server, "token", 1).await;

    mock_post_with_auth(
        &server,
        "/job/demo/buildWithParameters",
        Some("token"),
        Some("foo=bar"),
        ResponseTemplate::new(200).set_body_string("ok"),
        1,
    )
    .await;

    let client = JenkinsAsync::builder(server.uri())
        .auth_basic("user", "token")
        .with_crumb(Duration::from_secs(300))?
        .build()?;

    client
        .request(&TriggerBuild {
            job: "demo",
            params: &json!({ "foo": "bar" }),
        })
        .await?;

    server.verify().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_client_supports_base_path_with_crumb() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jenkins/crumbIssuer/api/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "crumbRequestField": "Jenkins-Crumb",
            "crumb": "token"
        })))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    mock_post_with_auth(
        &server,
        "/jenkins/job/demo/buildWithParameters",
        Some("token"),
        Some("foo=bar"),
        ResponseTemplate::new(200).set_body_string("ok"),
        1,
    )
    .await;

    let base_url = format!("{}/jenkins", server.uri());
    let client = JenkinsAsync::builder(base_url)
        .auth_basic("user", "token")
        .with_crumb(Duration::from_secs(300))?
        .build()?;

    let body = client
        .request(&TriggerBuild {
            job: "demo",
            params: &json!({ "foo": "bar" }),
        })
        .await?;
    assert_eq!(body, "ok");

    server.verify().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_retry_replays_on_server_errors() -> Result<()> {
    let server = MockServer::start().await;

    mock_get(&server, "/queue/api/json", ResponseTemplate::new(500), 1).await;

    mock_get(
        &server,
        "/queue/api/json",
        ResponseTemplate::new(200).set_body_json(json!({ "items": [] })),
        1,
    )
    .await;

    let client = JenkinsAsync::builder(server.uri())
        .with_retry(3, Duration::from_millis(5))
        .build()?;

    let response = client.request(&QueueLength).await?;
    assert!(response["items"].is_array());

    server.verify().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_jobs_info_uses_tree_query_param() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/json"))
        .and(query_param("tree", "jobs[name,url,color]"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jobs": []
        })))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let client = JenkinsAsync::builder(server.uri()).build()?;

    let jobs: serde_json::Value = client.request(&JobsInfo).await?;
    assert!(jobs["jobs"].is_array());

    server.verify().await;
    Ok(())
}

#[cfg(feature = "blocking-client")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blocking_client_reuses_crumb_with_retry() -> Result<()> {
    let server = MockServer::start().await;

    mock_crumb(&server, "token", 1).await;

    mock_post_with_auth(
        &server,
        "/job/demo/1/stop",
        Some("token"),
        None,
        ResponseTemplate::new(500),
        1,
    )
    .await;

    mock_post_with_auth(
        &server,
        "/job/demo/1/stop",
        Some("token"),
        None,
        ResponseTemplate::new(200).set_body_string("stopped"),
        1,
    )
    .await;

    let base_url = server.uri();
    task::spawn_blocking(move || -> Result<()> {
        let client = JenkinsBlocking::builder(base_url)
            .auth_basic("user", "token")
            .with_retry(3, Duration::from_millis(5))
            .with_crumb(Duration::from_secs(300))?
            .build()?;

        let body: String = client.request(&StopBuild {
            job: "demo",
            build: "1",
        })?;
        assert_eq!(body, "stopped");
        Ok(())
    })
    .await??;

    server.verify().await;
    Ok(())
}

#[cfg(feature = "blocking-client")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blocking_client_supports_base_path_with_crumb() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jenkins/crumbIssuer/api/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "crumbRequestField": "Jenkins-Crumb",
            "crumb": "token"
        })))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    mock_post_with_auth(
        &server,
        "/jenkins/job/demo/1/stop",
        Some("token"),
        None,
        ResponseTemplate::new(200).set_body_string("stopped"),
        1,
    )
    .await;

    let base_url = format!("{}/jenkins", server.uri());
    task::spawn_blocking(move || -> Result<()> {
        let client = JenkinsBlocking::builder(base_url)
            .auth_basic("user", "token")
            .with_crumb(Duration::from_secs(300))?
            .build()?;

        let body: String = client.request(&StopBuild {
            job: "demo",
            build: "1",
        })?;
        assert_eq!(body, "stopped");
        Ok(())
    })
    .await??;

    server.verify().await;
    Ok(())
}
