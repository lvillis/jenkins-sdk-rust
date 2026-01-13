use crate::transport::request::{Request, RequestBody};
use crate::{ComputerName, Error, ExecutorsInfo};
use http::HeaderValue;
use serde_json::Value;

/// Jenkins computers/nodes (core) APIs.
#[derive(Clone)]
#[cfg(feature = "async")]
pub struct ComputersService {
    client: crate::Client,
}

#[cfg(feature = "async")]
impl ComputersService {
    pub(crate) fn new(client: crate::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "async")]
impl ComputersService {
    /// `GET /computer/api/json`
    pub async fn list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["computer", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `POST /computer/doCreateItem?name=<name>` with XML body.
    pub async fn create_from_xml(
        &self,
        name: impl Into<ComputerName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        let req = Request::post(["computer", "doCreateItem"])
            .query_pair("name", name.as_str())
            .body(RequestBody::bytes_with_content_type(
                xml.into(),
                HeaderValue::from_static("application/xml"),
            ));
        self.client.send_unit(req).await
    }

    /// `POST /computer/doCreateItem?name=<to>&mode=copy&from=<from>`
    pub async fn copy(
        &self,
        from: impl Into<ComputerName>,
        to: impl Into<ComputerName>,
    ) -> Result<(), Error> {
        let from = from.into();
        let to = to.into();
        let req = Request::post(["computer", "doCreateItem"])
            .query_pair("name", to.as_str())
            .query_pair("mode", "copy")
            .query_pair("from", from.as_str());
        self.client.send_unit(req).await
    }

    /// `GET /computer/api/json` (typed subset).
    pub async fn executors_info(&self) -> Result<ExecutorsInfo, Error> {
        self.client
            .send_json(Request::get(["computer", "api", "json"]))
            .await
    }

    /// `GET /computer/<name>/api/json`
    pub async fn computer(
        &self,
        name: impl Into<ComputerName>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let name = name.into();
        let mut req = Request::get(["computer", name.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `POST /computer/<name>/toggleOffline`
    pub async fn toggle_offline(
        &self,
        name: impl Into<ComputerName>,
        offline_message: Option<&str>,
    ) -> Result<(), Error> {
        let name = name.into();
        let mut req = Request::post(["computer", name.as_str(), "toggleOffline"]);
        if let Some(message) = offline_message {
            req = req.query_pair("offlineMessage", message);
        }
        self.client.send_unit(req).await
    }

    /// `POST /computer/<name>/doDelete`
    pub async fn delete(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["computer", name.as_str(), "doDelete"]))
            .await
    }

    /// `GET /computer/<name>/config.xml`
    pub async fn get_config_xml(&self, name: impl Into<ComputerName>) -> Result<Vec<u8>, Error> {
        let name = name.into();
        self.client
            .send_bytes(Request::get(["computer", name.as_str(), "config.xml"]))
            .await
    }

    /// `POST /computer/<name>/config.xml` with XML body.
    pub async fn update_config_xml(
        &self,
        name: impl Into<ComputerName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(
                Request::post(["computer", name.as_str(), "config.xml"]).body(
                    RequestBody::bytes_with_content_type(
                        xml.into(),
                        HeaderValue::from_static("application/xml"),
                    ),
                ),
            )
            .await
    }

    /// `POST /computer/<name>/connect`
    pub async fn connect(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["computer", name.as_str(), "connect"]))
            .await
    }

    /// `POST /computer/<name>/disconnect`
    pub async fn disconnect(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["computer", name.as_str(), "disconnect"]))
            .await
    }

    /// `POST /computer/<name>/launchSlaveAgent`
    pub async fn launch_slave_agent(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post([
                "computer",
                name.as_str(),
                "launchSlaveAgent",
            ]))
            .await
    }
}

/// Jenkins computers/nodes (core) APIs (blocking).
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingComputersService {
    client: crate::BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingComputersService {
    pub(crate) fn new(client: crate::BlockingClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "blocking")]
impl BlockingComputersService {
    /// `GET /computer/api/json`
    pub fn list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["computer", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `POST /computer/doCreateItem?name=<name>` with XML body.
    pub fn create_from_xml(
        &self,
        name: impl Into<ComputerName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        let req = Request::post(["computer", "doCreateItem"])
            .query_pair("name", name.as_str())
            .body(RequestBody::bytes_with_content_type(
                xml.into(),
                HeaderValue::from_static("application/xml"),
            ));
        self.client.send_unit(req)
    }

    /// `POST /computer/doCreateItem?name=<to>&mode=copy&from=<from>`
    pub fn copy(
        &self,
        from: impl Into<ComputerName>,
        to: impl Into<ComputerName>,
    ) -> Result<(), Error> {
        let from = from.into();
        let to = to.into();
        let req = Request::post(["computer", "doCreateItem"])
            .query_pair("name", to.as_str())
            .query_pair("mode", "copy")
            .query_pair("from", from.as_str());
        self.client.send_unit(req)
    }

    /// `GET /computer/api/json` (typed subset).
    pub fn executors_info(&self) -> Result<ExecutorsInfo, Error> {
        self.client
            .send_json(Request::get(["computer", "api", "json"]))
    }

    /// `GET /computer/<name>/api/json`
    pub fn computer(
        &self,
        name: impl Into<ComputerName>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let name = name.into();
        let mut req = Request::get(["computer", name.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `POST /computer/<name>/toggleOffline`
    pub fn toggle_offline(
        &self,
        name: impl Into<ComputerName>,
        offline_message: Option<&str>,
    ) -> Result<(), Error> {
        let name = name.into();
        let mut req = Request::post(["computer", name.as_str(), "toggleOffline"]);
        if let Some(message) = offline_message {
            req = req.query_pair("offlineMessage", message);
        }
        self.client.send_unit(req)
    }

    /// `POST /computer/<name>/doDelete`
    pub fn delete(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["computer", name.as_str(), "doDelete"]))
    }

    /// `GET /computer/<name>/config.xml`
    pub fn get_config_xml(&self, name: impl Into<ComputerName>) -> Result<Vec<u8>, Error> {
        let name = name.into();
        self.client
            .send_bytes(Request::get(["computer", name.as_str(), "config.xml"]))
    }

    /// `POST /computer/<name>/config.xml` with XML body.
    pub fn update_config_xml(
        &self,
        name: impl Into<ComputerName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        self.client.send_unit(
            Request::post(["computer", name.as_str(), "config.xml"]).body(
                RequestBody::bytes_with_content_type(
                    xml.into(),
                    HeaderValue::from_static("application/xml"),
                ),
            ),
        )
    }

    /// `POST /computer/<name>/connect`
    pub fn connect(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["computer", name.as_str(), "connect"]))
    }

    /// `POST /computer/<name>/disconnect`
    pub fn disconnect(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["computer", name.as_str(), "disconnect"]))
    }

    /// `POST /computer/<name>/launchSlaveAgent`
    pub fn launch_slave_agent(&self, name: impl Into<ComputerName>) -> Result<(), Error> {
        let name = name.into();
        self.client.send_unit(Request::post([
            "computer",
            name.as_str(),
            "launchSlaveAgent",
        ]))
    }
}
