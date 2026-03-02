use crate::JobSummary;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// One view summary entry.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ViewSummary {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// View list payload (for example `GET /api/json?tree=views[name,url]`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ViewList {
    #[serde(default)]
    pub views: Vec<ViewSummary>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// View details payload (`GET /view/<name>/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ViewInfo {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub property: Vec<Value>,
    #[serde(default)]
    pub jobs: Vec<JobSummary>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
