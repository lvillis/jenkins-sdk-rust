use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// User payload from `GET /user/<id>/api/json`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UserInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub absolute_url: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub authorities: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
