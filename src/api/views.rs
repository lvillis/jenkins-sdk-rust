use crate::transport::request::{Request, RequestBody};
use crate::{Error, JobPath, ViewName};
use http::HeaderValue;
use serde_json::Value;

/// Jenkins views (core) APIs.
#[derive(Clone)]
#[cfg(feature = "async")]
pub struct ViewsService {
    client: crate::Client,
}

#[cfg(feature = "async")]
impl ViewsService {
    pub(crate) fn new(client: crate::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "async")]
impl ViewsService {
    /// `GET /api/json?tree=views[name,url]`
    pub async fn list(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["api", "json"]).query_pair("tree", "views[name,url]"))
            .await
    }

    /// `GET /view/<name>/api/json`
    pub async fn get(&self, name: impl Into<ViewName>, tree: Option<&str>) -> Result<Value, Error> {
        let name = name.into();
        let mut req = Request::get(["view", name.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `POST /createView?name=<name>` with XML body.
    pub async fn create_from_xml(
        &self,
        name: impl Into<ViewName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        let req = Request::post(["createView"])
            .query_pair("name", name.as_str())
            .body(RequestBody::bytes_with_content_type(
                xml.into(),
                HeaderValue::from_static("application/xml"),
            ));
        self.client.send_unit(req).await
    }

    /// `POST /view/<name>/doDelete`
    pub async fn delete(&self, name: impl Into<ViewName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["view", name.as_str(), "doDelete"]))
            .await
    }

    /// `POST /view/<name>/doRename?newName=<new_name>`
    pub async fn rename(
        &self,
        name: impl Into<ViewName>,
        new_name: impl Into<ViewName>,
    ) -> Result<(), Error> {
        let name = name.into();
        let new_name = new_name.into();
        let req = Request::post(["view", name.as_str(), "doRename"])
            .query_pair("newName", new_name.as_str());
        self.client.send_unit(req).await
    }

    /// `POST /view/<view>/addJobToView?name=<job>`
    pub async fn add_job(
        &self,
        view: impl Into<ViewName>,
        job: impl Into<JobPath>,
    ) -> Result<(), Error> {
        let view = view.into();
        let job = job.into();
        let req =
            Request::post(["view", view.as_str(), "addJobToView"]).query_pair("name", job.as_str());
        self.client.send_unit(req).await
    }

    /// `POST /view/<view>/removeJobFromView?name=<job>`
    pub async fn remove_job(
        &self,
        view: impl Into<ViewName>,
        job: impl Into<JobPath>,
    ) -> Result<(), Error> {
        let view = view.into();
        let job = job.into();
        let req = Request::post(["view", view.as_str(), "removeJobFromView"])
            .query_pair("name", job.as_str());
        self.client.send_unit(req).await
    }

    /// `GET /view/<name>/config.xml`
    pub async fn get_config_xml(&self, name: impl Into<ViewName>) -> Result<Vec<u8>, Error> {
        let name = name.into();
        self.client
            .send_bytes(Request::get(["view", name.as_str(), "config.xml"]))
            .await
    }

    /// `POST /view/<name>/config.xml` with XML body.
    pub async fn update_config_xml(
        &self,
        name: impl Into<ViewName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["view", name.as_str(), "config.xml"]).body(
                RequestBody::bytes_with_content_type(
                    xml.into(),
                    HeaderValue::from_static("application/xml"),
                ),
            ))
            .await
    }
}

/// Jenkins views (core) APIs (blocking).
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingViewsService {
    client: crate::BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingViewsService {
    pub(crate) fn new(client: crate::BlockingClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "blocking")]
impl BlockingViewsService {
    /// `GET /api/json?tree=views[name,url]`
    pub fn list(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["api", "json"]).query_pair("tree", "views[name,url]"))
    }

    /// `GET /view/<name>/api/json`
    pub fn get(&self, name: impl Into<ViewName>, tree: Option<&str>) -> Result<Value, Error> {
        let name = name.into();
        let mut req = Request::get(["view", name.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `POST /createView?name=<name>` with XML body.
    pub fn create_from_xml(
        &self,
        name: impl Into<ViewName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        let req = Request::post(["createView"])
            .query_pair("name", name.as_str())
            .body(RequestBody::bytes_with_content_type(
                xml.into(),
                HeaderValue::from_static("application/xml"),
            ));
        self.client.send_unit(req)
    }

    /// `POST /view/<name>/doDelete`
    pub fn delete(&self, name: impl Into<ViewName>) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["view", name.as_str(), "doDelete"]))
    }

    /// `POST /view/<name>/doRename?newName=<new_name>`
    pub fn rename(
        &self,
        name: impl Into<ViewName>,
        new_name: impl Into<ViewName>,
    ) -> Result<(), Error> {
        let name = name.into();
        let new_name = new_name.into();
        let req = Request::post(["view", name.as_str(), "doRename"])
            .query_pair("newName", new_name.as_str());
        self.client.send_unit(req)
    }

    /// `POST /view/<view>/addJobToView?name=<job>`
    pub fn add_job(&self, view: impl Into<ViewName>, job: impl Into<JobPath>) -> Result<(), Error> {
        let view = view.into();
        let job = job.into();
        let req =
            Request::post(["view", view.as_str(), "addJobToView"]).query_pair("name", job.as_str());
        self.client.send_unit(req)
    }

    /// `POST /view/<view>/removeJobFromView?name=<job>`
    pub fn remove_job(
        &self,
        view: impl Into<ViewName>,
        job: impl Into<JobPath>,
    ) -> Result<(), Error> {
        let view = view.into();
        let job = job.into();
        let req = Request::post(["view", view.as_str(), "removeJobFromView"])
            .query_pair("name", job.as_str());
        self.client.send_unit(req)
    }

    /// `GET /view/<name>/config.xml`
    pub fn get_config_xml(&self, name: impl Into<ViewName>) -> Result<Vec<u8>, Error> {
        let name = name.into();
        self.client
            .send_bytes(Request::get(["view", name.as_str(), "config.xml"]))
    }

    /// `POST /view/<name>/config.xml` with XML body.
    pub fn update_config_xml(
        &self,
        name: impl Into<ViewName>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let name = name.into();
        self.client
            .send_unit(Request::post(["view", name.as_str(), "config.xml"]).body(
                RequestBody::bytes_with_content_type(
                    xml.into(),
                    HeaderValue::from_static("application/xml"),
                ),
            ))
    }
}
