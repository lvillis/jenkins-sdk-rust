use crate::transport::request::Request;
use crate::{Error, QueueItemId};
use serde_json::Value;

/// Jenkins queue (core) APIs.
#[derive(Clone)]
#[cfg(feature = "async")]
pub struct QueueService {
    client: crate::Client,
}

#[cfg(feature = "async")]
impl QueueService {
    pub(crate) fn new(client: crate::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "async")]
impl QueueService {
    /// `GET /queue/api/json`
    pub async fn list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["queue", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `GET /queue/item/<id>/api/json`
    pub async fn item(
        &self,
        id: impl Into<QueueItemId>,
        tree: Option<&str>,
    ) -> Result<Value, Error> {
        let id = id.into();
        let mut req = Request::get(["queue", "item", id.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req).await
    }

    /// `POST /queue/cancelItem?id=<id>`
    pub async fn cancel(&self, id: impl Into<QueueItemId>) -> Result<(), Error> {
        let id = id.into();
        let req = Request::post(["queue", "cancelItem"]).query_pair("id", id.as_str());
        self.client.send_unit(req).await
    }
}

/// Jenkins queue (core) APIs (blocking).
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingQueueService {
    client: crate::BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingQueueService {
    pub(crate) fn new(client: crate::BlockingClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "blocking")]
impl BlockingQueueService {
    /// `GET /queue/api/json`
    pub fn list(&self, tree: Option<&str>) -> Result<Value, Error> {
        let mut req = Request::get(["queue", "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `GET /queue/item/<id>/api/json`
    pub fn item(&self, id: impl Into<QueueItemId>, tree: Option<&str>) -> Result<Value, Error> {
        let id = id.into();
        let mut req = Request::get(["queue", "item", id.as_str(), "api", "json"]);
        if let Some(tree) = tree {
            req = req.query_pair("tree", tree);
        }
        self.client.send_json(req)
    }

    /// `POST /queue/cancelItem?id=<id>`
    pub fn cancel(&self, id: impl Into<QueueItemId>) -> Result<(), Error> {
        let id = id.into();
        let req = Request::post(["queue", "cancelItem"]).query_pair("id", id.as_str());
        self.client.send_unit(req)
    }
}
