use std::time::Duration;

use anyhow::Result;
#[cfg(feature = "async")]
use http::StatusCode;
#[cfg(feature = "blocking")]
use jenkins_sdk::BlockingClient;
use jenkins_sdk::RetryConfig;
#[cfg(feature = "async")]
use jenkins_sdk::{Client, Error};
use serde_json::json;
#[cfg(feature = "blocking")]
use tokio::task;
#[cfg(feature = "async")]
use tokio::time::sleep;
use wiremock::matchers::query_param;
use wiremock::{
    Match, Mock, MockServer, Request, ResponseTemplate,
    matchers::{body_string_contains, header, method, path},
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

#[cfg(feature = "async")]
async fn mock_get(server: &MockServer, endpoint: &str, response: ResponseTemplate, expected: u64) {
    Mock::given(method("GET"))
        .and(path(endpoint))
        .respond_with(response)
        .expect(expected)
        .up_to_n_times(expected)
        .mount(server)
        .await;
}

#[cfg(feature = "async")]
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
        ResponseTemplate::new(201)
            .append_header("Location", "/queue/item/1/")
            .set_body_string("one"),
        1,
    )
    .await;

    mock_post_with_auth(
        &server,
        "/job/demo/buildWithParameters",
        Some("token-2"),
        None,
        ResponseTemplate::new(201)
            .append_header("Location", "/queue/item/2/")
            .set_body_string("two"),
        1,
    )
    .await;

    let client = Client::builder(server.uri())?
        .auth_basic("user", "token")
        .with_crumb(Duration::from_millis(0))
        .build()?;

    let first = client
        .jobs()
        .build_with_parameters("demo", [("foo", "bar")])
        .await?;
    assert_eq!(
        first.queue_item_id.as_ref().map(|id| id.as_str()),
        Some("1")
    );

    sleep(Duration::from_millis(5)).await;

    let second = client
        .jobs()
        .build_with_parameters("demo", [("foo", "baz")])
        .await?;
    assert_eq!(
        second.queue_item_id.as_ref().map(|id| id.as_str()),
        Some("2")
    );

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
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

    let client = Client::builder(server.uri())?.build()?;

    let err = client
        .queue()
        .list(None)
        .await
        .expect_err("expected HTTP error");

    match err {
        Error::NotFound(http) => {
            assert_eq!(http.status, StatusCode::NOT_FOUND);
            assert!(
                http.body_snippet
                    .as_deref()
                    .unwrap_or_default()
                    .contains("queue not found")
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_client_attaches_crumb_and_basic_auth() -> Result<()> {
    let server = MockServer::start().await;

    mock_crumb(&server, "token", 1).await;

    mock_post_with_auth(
        &server,
        "/job/demo/buildWithParameters",
        Some("token"),
        Some("foo=bar"),
        ResponseTemplate::new(201).append_header("Location", "/queue/item/1/"),
        1,
    )
    .await;

    let client = Client::builder(server.uri())?
        .auth_basic("user", "token")
        .with_crumb(Duration::from_secs(300))
        .build()?;

    client
        .jobs()
        .build_with_parameters("demo", [("foo", "bar")])
        .await?;

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
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
        ResponseTemplate::new(201).append_header("Location", "/jenkins/queue/item/3/"),
        1,
    )
    .await;

    let base_url = format!("{}/jenkins", server.uri());
    let client = Client::builder(base_url)?
        .auth_basic("user", "token")
        .with_crumb(Duration::from_secs(300))
        .build()?;

    let triggered = client
        .jobs()
        .build_with_parameters("demo", [("foo", "bar")])
        .await?;
    assert_eq!(
        triggered.queue_item_id.as_ref().map(|id| id.as_str()),
        Some("3")
    );

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_retry_replays_on_server_errors() -> Result<()> {
    let server = MockServer::start().await;

    mock_get(&server, "/queue/api/json", ResponseTemplate::new(503), 1).await;

    mock_get(
        &server,
        "/queue/api/json",
        ResponseTemplate::new(200).set_body_json(json!({ "items": [] })),
        1,
    )
    .await;

    let client = Client::builder(server.uri())?
        .with_retry(3, Duration::from_millis(5))
        .build()?;

    let response = client.queue().list(None).await?;
    assert!(response["items"].is_array());

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_retry_replays_on_rate_limited_with_retry_after() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/queue/api/json"))
        .respond_with(ResponseTemplate::new(429).append_header("Retry-After", "0"))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    mock_get(
        &server,
        "/queue/api/json",
        ResponseTemplate::new(200).set_body_json(json!({ "items": [] })),
        1,
    )
    .await;

    let client = Client::builder(server.uri())?
        .retry_config(RetryConfig {
            max_retries: 1,
            base_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
            jitter: false,
            retry_non_idempotent: false,
            respect_retry_after: true,
        })
        .build()?;

    let response = client.queue().list(None).await?;
    assert!(response["items"].is_array());

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_error_body_snippet_redacts_auth_token() -> Result<()> {
    let server = MockServer::start().await;

    mock_get(
        &server,
        "/queue/api/json",
        ResponseTemplate::new(500).set_body_string("supersecret"),
        1,
    )
    .await;

    let client = Client::builder(server.uri())?
        .auth_basic("user", "supersecret")
        .build()?;

    let err = client
        .queue()
        .list(None)
        .await
        .expect_err("expected HTTP error");

    match err {
        Error::Api(http) => {
            let snippet = http.body_snippet.as_deref().unwrap_or_default();
            assert!(!snippet.contains("supersecret"));
            assert!(snippet.contains("<redacted>"));
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
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

    let client = Client::builder(server.uri())?.build()?;

    let jobs: serde_json::Value = client.jobs().list().await?;
    assert!(jobs["jobs"].is_array());

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_jobs_progressive_console_text_parses_headers() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/job/demo/1/logText/progressiveText"))
        .and(query_param("start", "0"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("X-Text-Size", "5")
                .append_header("X-More-Data", "true")
                .set_body_string("hello"),
        )
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let client = Client::builder(server.uri())?.build()?;

    let chunk = client
        .jobs()
        .progressive_console_text("demo", "1", 0)
        .await?;
    assert_eq!(chunk.text, "hello");
    assert_eq!(chunk.next_start, Some(5));
    assert!(chunk.more_data);

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_jobs_download_artifact_splits_path_segments() -> Result<()> {
    let server = MockServer::start().await;

    mock_get(
        &server,
        "/job/demo/1/artifact/a/b%20c.txt",
        ResponseTemplate::new(200).set_body_bytes(vec![1, 2, 3]),
        1,
    )
    .await;

    let client = Client::builder(server.uri())?.build()?;

    let bytes = client
        .jobs()
        .download_artifact("demo", "1", "a/b c.txt")
        .await?;
    assert_eq!(bytes, vec![1, 2, 3]);

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_system_downloads_jnlp_jars() -> Result<()> {
    let server = MockServer::start().await;

    mock_get(
        &server,
        "/jnlpJars/agent.jar",
        ResponseTemplate::new(200).set_body_bytes(vec![1]),
        1,
    )
    .await;
    mock_get(
        &server,
        "/jnlpJars/jenkins-cli.jar",
        ResponseTemplate::new(200).set_body_bytes(vec![2]),
        1,
    )
    .await;

    let client = Client::builder(server.uri())?.build()?;

    assert_eq!(client.system().agent_jar().await?, vec![1]);
    assert_eq!(client.system().cli_jar().await?, vec![2]);

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_computers_create_from_xml_posts_xml() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/computer/doCreateItem"))
        .and(query_param("name", "agent-1"))
        .and(header("Content-Type", "application/xml"))
        .and(body_string_contains("<slave/>"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let client = Client::builder(server.uri())?.build()?;
    client
        .computers()
        .create_from_xml("agent-1", "<slave/>")
        .await?;

    server.verify().await;
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_users_config_xml_supports_get_and_update() -> Result<()> {
    let server = MockServer::start().await;

    mock_get(
        &server,
        "/user/alice/config.xml",
        ResponseTemplate::new(200).set_body_string("<u/>"),
        1,
    )
    .await;

    Mock::given(method("POST"))
        .and(path("/user/alice/config.xml"))
        .and(header("Content-Type", "application/xml"))
        .and(body_string_contains("<u/>"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let client = Client::builder(server.uri())?.build()?;

    let xml = client.users().get_config_xml("alice").await?;
    assert_eq!(String::from_utf8_lossy(&xml), "<u/>");

    client.users().update_config_xml("alice", xml).await?;

    server.verify().await;
    Ok(())
}

#[cfg(feature = "blocking")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blocking_client_reuses_crumb_with_retry() -> Result<()> {
    let server = MockServer::start().await;

    mock_crumb(&server, "token", 1).await;

    mock_post_with_auth(
        &server,
        "/job/demo/1/stop",
        Some("token"),
        None,
        ResponseTemplate::new(503),
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
        let client = BlockingClient::builder(base_url)?
            .auth_basic("user", "token")
            .retry_config(RetryConfig {
                max_retries: 3,
                base_delay: Duration::from_millis(5),
                max_delay: Duration::from_millis(5),
                jitter: false,
                retry_non_idempotent: true,
                respect_retry_after: true,
            })
            .with_crumb(Duration::from_secs(300))
            .build()?;

        client.jobs().stop_build("demo", "1")?;
        Ok(())
    })
    .await??;

    server.verify().await;
    Ok(())
}

#[cfg(feature = "blocking")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blocking_jobs_progressive_console_text_parses_headers() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/job/demo/1/logText/progressiveText"))
        .and(query_param("start", "0"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("X-Text-Size", "5")
                .append_header("X-More-Data", "true")
                .set_body_string("hello"),
        )
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    task::spawn_blocking(move || -> Result<()> {
        let client = BlockingClient::builder(base_url)?.build()?;

        let chunk = client.jobs().progressive_console_text("demo", "1", 0)?;
        assert_eq!(chunk.text, "hello");
        assert_eq!(chunk.next_start, Some(5));
        assert!(chunk.more_data);

        Ok(())
    })
    .await??;

    server.verify().await;
    Ok(())
}

#[cfg(feature = "blocking")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blocking_jobs_download_artifact_splits_path_segments() -> Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/job/demo/1/artifact/a/b%20c.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1, 2, 3]))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    task::spawn_blocking(move || -> Result<()> {
        let client = BlockingClient::builder(base_url)?.build()?;

        let bytes = client.jobs().download_artifact("demo", "1", "a/b c.txt")?;
        assert_eq!(bytes, vec![1, 2, 3]);

        Ok(())
    })
    .await??;

    server.verify().await;
    Ok(())
}

#[cfg(feature = "blocking")]
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
        let client = BlockingClient::builder(base_url)?
            .auth_basic("user", "token")
            .with_crumb(Duration::from_secs(300))
            .build()?;

        client.jobs().stop_build("demo", "1")?;
        Ok(())
    })
    .await??;

    server.verify().await;
    Ok(())
}
