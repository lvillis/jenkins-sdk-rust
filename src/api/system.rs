use crate::transport::request::{Request, RequestBody};
use crate::{Crumb, Error};
use http::HeaderValue;
use serde_json::Value;

/// Jenkins system-level (core) APIs.
#[derive(Clone)]
#[cfg(feature = "async")]
pub struct SystemService {
    client: crate::Client,
}

#[cfg(feature = "async")]
impl SystemService {
    pub(crate) fn new(client: crate::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "async")]
impl SystemService {
    /// `GET /api/json`
    pub async fn root(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `GET /overallLoad/api/json`
    pub async fn overall_load(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["overallLoad", "api", "json"]))
            .await
    }

    /// `GET /loadStatistics/api/json`
    pub async fn load_statistics(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["loadStatistics", "api", "json"]))
            .await
    }

    /// `GET /whoAmI/api/json`
    pub async fn who_am_i(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["whoAmI", "api", "json"]))
            .await
    }

    /// `GET /crumbIssuer/api/json`
    pub async fn crumb(&self) -> Result<Crumb, Error> {
        self.client
            .send_json(Request::get(["crumbIssuer", "api", "json"]))
            .await
    }

    /// `GET /jnlpJars/agent.jar`
    pub async fn agent_jar(&self) -> Result<Vec<u8>, Error> {
        self.client
            .send_bytes(Request::get(["jnlpJars", "agent.jar"]))
            .await
    }

    /// `GET /jnlpJars/jenkins-cli.jar`
    pub async fn cli_jar(&self) -> Result<Vec<u8>, Error> {
        self.client
            .send_bytes(Request::get(["jnlpJars", "jenkins-cli.jar"]))
            .await
    }

    /// `GET /config.xml`
    pub async fn get_config_xml(&self) -> Result<Vec<u8>, Error> {
        self.client.send_bytes(Request::get(["config.xml"])).await
    }

    /// `POST /config.xml` with XML body.
    pub async fn update_config_xml(&self, xml: impl Into<Vec<u8>>) -> Result<(), Error> {
        self.client
            .send_unit(
                Request::post(["config.xml"]).body(RequestBody::bytes_with_content_type(
                    xml.into(),
                    HeaderValue::from_static("application/xml"),
                )),
            )
            .await
    }

    /// `POST /quietDown`
    pub async fn quiet_down(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["quietDown"])).await
    }

    /// `POST /cancelQuietDown`
    pub async fn cancel_quiet_down(&self) -> Result<(), Error> {
        self.client
            .send_unit(Request::post(["cancelQuietDown"]))
            .await
    }

    /// `POST /reload`
    pub async fn reload_configuration(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["reload"])).await
    }

    /// `POST /safeRestart`
    pub async fn safe_restart(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["safeRestart"])).await
    }

    /// `POST /restart`
    pub async fn restart(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["restart"])).await
    }

    /// `POST /exit`
    pub async fn exit(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["exit"])).await
    }
}

/// Jenkins system-level (core) APIs (blocking).
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingSystemService {
    client: crate::BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingSystemService {
    pub(crate) fn new(client: crate::BlockingClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "blocking")]
impl BlockingSystemService {
    /// `GET /api/json`
    pub fn root(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `GET /overallLoad/api/json`
    pub fn overall_load(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["overallLoad", "api", "json"]))
    }

    /// `GET /loadStatistics/api/json`
    pub fn load_statistics(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["loadStatistics", "api", "json"]))
    }

    /// `GET /whoAmI/api/json`
    pub fn who_am_i(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["whoAmI", "api", "json"]))
    }

    /// `GET /crumbIssuer/api/json`
    pub fn crumb(&self) -> Result<Crumb, Error> {
        self.client
            .send_json(Request::get(["crumbIssuer", "api", "json"]))
    }

    /// `GET /jnlpJars/agent.jar`
    pub fn agent_jar(&self) -> Result<Vec<u8>, Error> {
        self.client
            .send_bytes(Request::get(["jnlpJars", "agent.jar"]))
    }

    /// `GET /jnlpJars/jenkins-cli.jar`
    pub fn cli_jar(&self) -> Result<Vec<u8>, Error> {
        self.client
            .send_bytes(Request::get(["jnlpJars", "jenkins-cli.jar"]))
    }

    /// `GET /config.xml`
    pub fn get_config_xml(&self) -> Result<Vec<u8>, Error> {
        self.client.send_bytes(Request::get(["config.xml"]))
    }

    /// `POST /config.xml` with XML body.
    pub fn update_config_xml(&self, xml: impl Into<Vec<u8>>) -> Result<(), Error> {
        self.client.send_unit(Request::post(["config.xml"]).body(
            RequestBody::bytes_with_content_type(
                xml.into(),
                HeaderValue::from_static("application/xml"),
            ),
        ))
    }

    /// `POST /quietDown`
    pub fn quiet_down(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["quietDown"]))
    }

    /// `POST /cancelQuietDown`
    pub fn cancel_quiet_down(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["cancelQuietDown"]))
    }

    /// `POST /reload`
    pub fn reload_configuration(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["reload"]))
    }

    /// `POST /safeRestart`
    pub fn safe_restart(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["safeRestart"]))
    }

    /// `POST /restart`
    pub fn restart(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["restart"]))
    }

    /// `POST /exit`
    pub fn exit(&self) -> Result<(), Error> {
        self.client.send_unit(Request::post(["exit"]))
    }
}
