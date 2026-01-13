use crate::transport::request::{Request, RequestBody};
use crate::{Error, UserId};
use http::HeaderValue;
use serde_json::Value;

/// Jenkins users (core) APIs.
#[derive(Clone)]
#[cfg(feature = "async")]
pub struct UsersService {
    client: crate::Client,
}

#[cfg(feature = "async")]
impl UsersService {
    pub(crate) fn new(client: crate::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "async")]
impl UsersService {
    /// `GET /user/<id>/api/json`
    pub async fn get(&self, id: impl Into<UserId>, tree: Option<&str>) -> Result<Value, Error> {
        let id = id.into();
        let mut req = Request::get(["user", id.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `GET /whoAmI/api/json`
    pub async fn who_am_i(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["whoAmI", "api", "json"]))
            .await
    }

    /// `GET /user/<id>/config.xml`
    pub async fn get_config_xml(&self, id: impl Into<UserId>) -> Result<Vec<u8>, Error> {
        let id = id.into();
        self.client
            .send_bytes(Request::get(["user", id.as_str(), "config.xml"]))
            .await
    }

    /// `POST /user/<id>/config.xml` with XML body.
    pub async fn update_config_xml(
        &self,
        id: impl Into<UserId>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let id = id.into();
        self.client
            .send_unit(Request::post(["user", id.as_str(), "config.xml"]).body(
                RequestBody::bytes_with_content_type(
                    xml.into(),
                    HeaderValue::from_static("application/xml"),
                ),
            ))
            .await
    }
}

/// Jenkins users (core) APIs (blocking).
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingUsersService {
    client: crate::BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingUsersService {
    pub(crate) fn new(client: crate::BlockingClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "blocking")]
impl BlockingUsersService {
    /// `GET /user/<id>/api/json`
    pub fn get(&self, id: impl Into<UserId>, tree: Option<&str>) -> Result<Value, Error> {
        let id = id.into();
        let mut req = Request::get(["user", id.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `GET /whoAmI/api/json`
    pub fn who_am_i(&self) -> Result<Value, Error> {
        self.client
            .send_json(Request::get(["whoAmI", "api", "json"]))
    }

    /// `GET /user/<id>/config.xml`
    pub fn get_config_xml(&self, id: impl Into<UserId>) -> Result<Vec<u8>, Error> {
        let id = id.into();
        self.client
            .send_bytes(Request::get(["user", id.as_str(), "config.xml"]))
    }

    /// `POST /user/<id>/config.xml` with XML body.
    pub fn update_config_xml(
        &self,
        id: impl Into<UserId>,
        xml: impl Into<Vec<u8>>,
    ) -> Result<(), Error> {
        let id = id.into();
        self.client
            .send_unit(Request::post(["user", id.as_str(), "config.xml"]).body(
                RequestBody::bytes_with_content_type(
                    xml.into(),
                    HeaderValue::from_static("application/xml"),
                ),
            ))
    }
}
