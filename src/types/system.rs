use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// JSON payload of `GET /crumbIssuer/api/json`.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Crumb {
    #[serde(rename = "crumbRequestField")]
    pub crumb_request_field: String,
    pub crumb: String,
}

/// Root payload of `GET /api/json`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SystemRoot {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub node_name: Option<String>,
    #[serde(default)]
    pub node_description: Option<String>,
    #[serde(default)]
    pub num_executors: Option<u32>,
    #[serde(default)]
    pub quieting_down: Option<bool>,
    #[serde(default)]
    pub use_crumbs: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Payload of `GET /whoAmI/api/json`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WhoAmI {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub anonymous: Option<bool>,
    #[serde(default)]
    pub authenticated: Option<bool>,
    #[serde(default)]
    pub authorities: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Generic system payload for less stable Jenkins core structures.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SystemPayload {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
