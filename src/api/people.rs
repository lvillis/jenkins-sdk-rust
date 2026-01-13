use crate::Error;
use crate::transport::request::Request;
use serde_json::Value;

/// Jenkins people APIs (core).
#[derive(Clone)]
#[cfg(feature = "async")]
pub struct PeopleService {
    client: crate::Client,
}

#[cfg(feature = "async")]
impl PeopleService {
    pub(crate) fn new(client: crate::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "async")]
impl PeopleService {
    /// `GET /people/api/json`
    pub async fn list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["people", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `GET /asynchPeople/api/json`
    pub async fn asynch_list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["asynchPeople", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }
}

/// Jenkins people APIs (core) (blocking).
#[derive(Clone)]
#[cfg(feature = "blocking")]
pub struct BlockingPeopleService {
    client: crate::BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingPeopleService {
    pub(crate) fn new(client: crate::BlockingClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "blocking")]
impl BlockingPeopleService {
    /// `GET /people/api/json`
    pub fn list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["people", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `GET /asynchPeople/api/json`
    pub fn asynch_list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["asynchPeople", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }
}
