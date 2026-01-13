use crate::transport::request::{Request, RequestBody, Response};
use crate::{
    ArtifactPath, BuildNumber, Error, JobName, JobPath, ProgressiveText, QueueItemId,
    TriggeredBuild,
};
use http::HeaderValue;
use serde_json::Value;

fn job_segments(job: &JobPath) -> Vec<String> {
    job.url_segments().map(ToOwned::to_owned).collect()
}

fn build_selector_request(job: &JobPath, selector: &str, tree: Option<&str>) -> Request {
    let mut segments = job_segments(job);
    segments.push(selector.to_owned());
    segments.extend(["api", "json"].map(str::to_owned));

    let mut req = Request::get(segments);
    if let Some(tree) = tree {
        req = req.query_pair("tree", tree);
    }
    req
}

fn parse_progressive_text(resp: Response) -> ProgressiveText {
    let next_start = resp
        .headers
        .get("X-Text-Size")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    let more_data = resp
        .headers
        .get("X-More-Data")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| s.eq_ignore_ascii_case("true"));
    let text = String::from_utf8_lossy(&resp.body).into_owned();
    ProgressiveText {
        text,
        next_start,
        more_data,
    }
}

fn parse_queue_item_id_from_location(location: &str) -> Option<QueueItemId> {
    let segments: Vec<&str> = location.split('/').filter(|s| !s.is_empty()).collect();
    let item_pos = segments.iter().position(|s| *s == "item")?;
    let id = segments.get(item_pos + 1)?;
    Some(QueueItemId::new(*id))
}

fn triggered_build_from_response(resp: &Response) -> TriggeredBuild {
    let location = resp
        .headers
        .get(http::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string().into_boxed_str());
    let queue_item_id = location
        .as_deref()
        .and_then(parse_queue_item_id_from_location);
    TriggeredBuild {
        queue_item_id,
        location,
    }
}

/// Jenkins jobs/builds (core) APIs.
#[derive(Clone)]
#[cfg(feature = "async")]
pub struct JobsService {
    client: crate::Client,
}

#[cfg(feature = "async")]
impl JobsService {
    pub(crate) fn new(client: crate::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "async")]
impl JobsService {
    /// `GET /api/json?tree=jobs[name,url,color]`
    pub async fn list(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["api", "json"]).query_pair("tree", "jobs[name,url,color]"))
            .await
    }

    /// `GET /job/<name>/api/json`
    pub async fn get(&self, job: impl Into<JobPath>, tree: Option<&str>) -> Result<Value, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.extend(["api", "json"].map(str::to_owned));

        let mut req = Request::get(segments);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `GET /job/<name>/lastBuild/api/json`
    pub async fn last_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.extend(["lastBuild", "api", "json"].map(str::to_owned));

        let mut req = Request::get(segments);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `GET /job/<name>/lastCompletedBuild/api/json`
    pub async fn last_completed_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastCompletedBuild", tree))
            .await
    }

    /// `GET /job/<name>/lastSuccessfulBuild/api/json`
    pub async fn last_successful_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastSuccessfulBuild", tree))
            .await
    }

    /// `GET /job/<name>/lastFailedBuild/api/json`
    pub async fn last_failed_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastFailedBuild", tree))
            .await
    }

    /// `GET /job/<name>/lastStableBuild/api/json`
    pub async fn last_stable_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastStableBuild", tree))
            .await
    }

    /// `GET /job/<name>/lastUnstableBuild/api/json`
    pub async fn last_unstable_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastUnstableBuild", tree))
            .await
    }

    /// `GET /job/<name>/lastUnsuccessfulBuild/api/json`
    pub async fn last_unsuccessful_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastUnsuccessfulBuild", tree))
            .await
    }

    /// `GET /job/<name>/<build>/logText/progressiveText?start=<offset>`
    pub async fn progressive_console_text(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        start: u64,
    ) -> Result<ProgressiveText, Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.extend(["logText", "progressiveText"].map(str::to_owned));
        let req = Request::get(segments).query_pair("start", start.to_string());

        let resp = self.client.send_response(req).await?;
        Ok(parse_progressive_text(resp))
    }

    /// `GET /job/<name>/<build>/api/json`
    pub async fn build_info(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.extend(["api", "json"].map(str::to_owned));

        let mut req = Request::get(segments);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `GET /job/<name>/lastBuild/consoleText`
    pub async fn last_console_text(&self, job: impl Into<JobPath>) -> Result<String, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.extend(["lastBuild", "consoleText"].map(str::to_owned));
        self.client.send_text(Request::get(segments)).await
    }

    /// `GET /job/<name>/<build>/consoleText`
    pub async fn console_text(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<String, Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("consoleText".to_owned());
        self.client.send_text(Request::get(segments)).await
    }

    /// `GET /job/<name>/<build>/artifact/<path>`
    pub async fn download_artifact(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        path: impl Into<ArtifactPath>,
    ) -> Result<Vec<u8>, Error> {
        let job = job.into();
        let build = build.into();
        let path = path.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("artifact".to_owned());
        segments.extend(path.url_segments().map(ToOwned::to_owned));
        self.client.send_bytes(Request::get(segments)).await
    }

    /// `POST /job/<name>/<build>/stop`
    pub async fn stop_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("stop".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/<build>/term`
    pub async fn term_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("term".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/<build>/kill`
    pub async fn kill_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("kill".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/<build>/doDelete`
    pub async fn delete_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("doDelete".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/<build>/toggleLogKeep`
    pub async fn toggle_keep_log(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("toggleLogKeep".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/<build>/submitDescription`
    pub async fn set_build_description(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let description = description.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("submitDescription".to_owned());
        let req = Request::post(segments).form_pairs([("description", description)]);
        self.client.send_unit(req).await
    }

    /// `POST /job/<name>/submitDescription`
    pub async fn set_job_description(
        &self,
        job: impl Into<JobPath>,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        let job = job.into();
        let description = description.into();

        let mut segments = job_segments(&job);
        segments.push("submitDescription".to_owned());
        let req = Request::post(segments).form_pairs([("description", description)]);
        self.client.send_unit(req).await
    }

    /// `POST /job/<name>/build`
    pub async fn build(&self, job: impl Into<JobPath>) -> Result<TriggeredBuild, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("build".to_owned());
        let resp = self.client.send_response(Request::post(segments)).await?;
        Ok(triggered_build_from_response(&resp))
    }

    /// `POST /job/<name>/buildWithParameters`
    pub async fn build_with_parameters<I, K, V>(
        &self,
        job: impl Into<JobPath>,
        params: I,
    ) -> Result<TriggeredBuild, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("buildWithParameters".to_owned());
        let req = Request::post(segments).form_pairs(params);
        let resp = self.client.send_response(req).await?;
        Ok(triggered_build_from_response(&resp))
    }

    /// `GET /job/<name>/config.xml`
    pub async fn get_config_xml(&self, job: impl Into<JobPath>) -> Result<Vec<u8>, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("config.xml".to_owned());
        self.client.send_bytes(Request::get(segments)).await
    }

    /// `POST /job/<name>/config.xml` with XML body.
    pub async fn update_config_xml(
        &self,
        job: impl Into<JobPath>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("config.xml".to_owned());
        let req = Request::post(segments).body(RequestBody::bytes_with_content_type(
            xml.into(),
            HeaderValue::from_static("application/xml"),
        ));
        self.client.send_unit(req).await
    }

    /// `POST /createItem?name=<name>` with XML body.
    pub async fn create_from_xml(
        &self,
        name: impl Into<JobName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        let req = Request::post(["createItem"])
            .query_pair("name", name.as_str())
            .body(RequestBody::bytes_with_content_type(
                xml.into(),
                HeaderValue::from_static("application/xml"),
            ));
        self.client.send_unit(req).await
    }

    /// `POST /createItem?name=<new>&mode=copy&from=<from>`
    pub async fn copy(
        &self,
        from: impl Into<JobPath>,
        to: impl Into<JobName>,
    ) -> Result<(), Error> {
        let from = from.into();
        let to = to.into();
        let req = Request::post(["createItem"])
            .query_pair("name", to.as_str())
            .query_pair("mode", "copy")
            .query_pair("from", from.as_str());
        self.client.send_unit(req).await
    }

    /// `POST /job/<name>/doDelete`
    pub async fn delete(&self, job: impl Into<JobPath>) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("doDelete".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/disable`
    pub async fn disable(&self, job: impl Into<JobPath>) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("disable".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/enable`
    pub async fn enable(&self, job: impl Into<JobPath>) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("enable".to_owned());
        self.client.send_unit(Request::post(segments)).await
    }

    /// `POST /job/<name>/doRename?newName=<new_name>`
    pub async fn rename(
        &self,
        job: impl Into<JobPath>,
        new_name: impl Into<JobName>,
    ) -> Result<(), Error> {
        let job = job.into();
        let new_name = new_name.into();
        let mut segments = job_segments(&job);
        segments.push("doRename".to_owned());
        let req = Request::post(segments).query_pair("newName", new_name.as_str());
        self.client.send_unit(req).await
    }
}

/// Jenkins jobs/builds (core) APIs (blocking).
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingJobsService {
    client: crate::BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingJobsService {
    pub(crate) fn new(client: crate::BlockingClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "blocking")]
impl BlockingJobsService {
    /// `GET /api/json?tree=jobs[name,url,color]`
    pub fn list(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["api", "json"]).query_pair("tree", "jobs[name,url,color]"))
    }

    /// `GET /job/<name>/api/json`
    pub fn get(&self, job: impl Into<JobPath>, tree: Option<&str>) -> Result<Value, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.extend(["api", "json"].map(str::to_owned));

        let mut req = Request::get(segments);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `GET /job/<name>/lastBuild/api/json`
    pub fn last_build(&self, job: impl Into<JobPath>, tree: Option<&str>) -> Result<Value, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.extend(["lastBuild", "api", "json"].map(str::to_owned));

        let mut req = Request::get(segments);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `GET /job/<name>/lastCompletedBuild/api/json`
    pub fn last_completed_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastCompletedBuild", tree))
    }

    /// `GET /job/<name>/lastSuccessfulBuild/api/json`
    pub fn last_successful_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastSuccessfulBuild", tree))
    }

    /// `GET /job/<name>/lastFailedBuild/api/json`
    pub fn last_failed_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastFailedBuild", tree))
    }

    /// `GET /job/<name>/lastStableBuild/api/json`
    pub fn last_stable_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastStableBuild", tree))
    }

    /// `GET /job/<name>/lastUnstableBuild/api/json`
    pub fn last_unstable_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastUnstableBuild", tree))
    }

    /// `GET /job/<name>/lastUnsuccessfulBuild/api/json`
    pub fn last_unsuccessful_build(
        &self,
        job: impl Into<JobPath>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        self.client
            .send_json(build_selector_request(&job, "lastUnsuccessfulBuild", tree))
    }

    /// `GET /job/<name>/<build>/logText/progressiveText?start=<offset>`
    pub fn progressive_console_text(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        start: u64,
    ) -> Result<ProgressiveText, Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.extend(["logText", "progressiveText"].map(str::to_owned));
        let req = Request::get(segments).query_pair("start", start.to_string());

        let resp = self.client.send_response(req)?;
        Ok(parse_progressive_text(resp))
    }

    /// `GET /job/<name>/<build>/api/json`
    pub fn build_info(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.extend(["api", "json"].map(str::to_owned));

        let mut req = Request::get(segments);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `GET /job/<name>/lastBuild/consoleText`
    pub fn last_console_text(&self, job: impl Into<JobPath>) -> Result<String, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.extend(["lastBuild", "consoleText"].map(str::to_owned));
        self.client.send_text(Request::get(segments))
    }

    /// `GET /job/<name>/<build>/consoleText`
    pub fn console_text(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<String, Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("consoleText".to_owned());
        self.client.send_text(Request::get(segments))
    }

    /// `GET /job/<name>/<build>/artifact/<path>`
    pub fn download_artifact(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        path: impl Into<ArtifactPath>,
    ) -> Result<Vec<u8>, Error> {
        let job = job.into();
        let build = build.into();
        let path = path.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("artifact".to_owned());
        segments.extend(path.url_segments().map(ToOwned::to_owned));
        self.client.send_bytes(Request::get(segments))
    }

    /// `POST /job/<name>/<build>/stop`
    pub fn stop_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("stop".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/<build>/term`
    pub fn term_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("term".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/<build>/kill`
    pub fn kill_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("kill".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/<build>/doDelete`
    pub fn delete_build(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("doDelete".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/<build>/toggleLogKeep`
    pub fn toggle_keep_log(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("toggleLogKeep".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/<build>/submitDescription`
    pub fn set_build_description(
        &self,
        job: impl Into<JobPath>,
        build: impl Into<BuildNumber>,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        let job = job.into();
        let build = build.into();
        let description = description.into();

        let mut segments = job_segments(&job);
        segments.push(build.as_str().to_owned());
        segments.push("submitDescription".to_owned());
        let req = Request::post(segments).form_pairs([("description", description)]);
        self.client.send_unit(req)
    }

    /// `POST /job/<name>/submitDescription`
    pub fn set_job_description(
        &self,
        job: impl Into<JobPath>,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        let job = job.into();
        let description = description.into();

        let mut segments = job_segments(&job);
        segments.push("submitDescription".to_owned());
        let req = Request::post(segments).form_pairs([("description", description)]);
        self.client.send_unit(req)
    }

    /// `POST /job/<name>/build`
    pub fn build(&self, job: impl Into<JobPath>) -> Result<TriggeredBuild, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("build".to_owned());
        let resp = self.client.send_response(Request::post(segments))?;
        Ok(triggered_build_from_response(&resp))
    }

    /// `POST /job/<name>/buildWithParameters`
    pub fn build_with_parameters<I, K, V>(
        &self,
        job: impl Into<JobPath>,
        params: I,
    ) -> Result<TriggeredBuild, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("buildWithParameters".to_owned());
        let req = Request::post(segments).form_pairs(params);
        let resp = self.client.send_response(req)?;
        Ok(triggered_build_from_response(&resp))
    }

    /// `GET /job/<name>/config.xml`
    pub fn get_config_xml(&self, job: impl Into<JobPath>) -> Result<Vec<u8>, Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("config.xml".to_owned());
        self.client.send_bytes(Request::get(segments))
    }

    /// `POST /job/<name>/config.xml` with XML body.
    pub fn update_config_xml(
        &self,
        job: impl Into<JobPath>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("config.xml".to_owned());
        let req = Request::post(segments).body(RequestBody::bytes_with_content_type(
            xml.into(),
            HeaderValue::from_static("application/xml"),
        ));
        self.client.send_unit(req)
    }

    /// `POST /createItem?name=<name>` with XML body.
    pub fn create_from_xml(
        &self,
        name: impl Into<JobName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        let req = Request::post(["createItem"])
            .query_pair("name", name.as_str())
            .body(RequestBody::bytes_with_content_type(
                xml.into(),
                HeaderValue::from_static("application/xml"),
            ));
        self.client.send_unit(req)
    }

    /// `POST /createItem?name=<new>&mode=copy&from=<from>`
    pub fn copy(&self, from: impl Into<JobPath>, to: impl Into<JobName>) -> Result<(), Error> {
        let from = from.into();
        let to = to.into();
        let req = Request::post(["createItem"])
            .query_pair("name", to.as_str())
            .query_pair("mode", "copy")
            .query_pair("from", from.as_str());
        self.client.send_unit(req)
    }

    /// `POST /job/<name>/doDelete`
    pub fn delete(&self, job: impl Into<JobPath>) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("doDelete".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/disable`
    pub fn disable(&self, job: impl Into<JobPath>) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("disable".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/enable`
    pub fn enable(&self, job: impl Into<JobPath>) -> Result<(), Error> {
        let job = job.into();
        let mut segments = job_segments(&job);
        segments.push("enable".to_owned());
        self.client.send_unit(Request::post(segments))
    }

    /// `POST /job/<name>/doRename?newName=<new_name>`
    pub fn rename(
        &self,
        job: impl Into<JobPath>,
        new_name: impl Into<JobName>,
    ) -> Result<(), Error> {
        let job = job.into();
        let new_name = new_name.into();
        let mut segments = job_segments(&job);
        segments.push("doRename".to_owned());
        let req = Request::post(segments).query_pair("newName", new_name.as_str());
        self.client.send_unit(req)
    }
}
