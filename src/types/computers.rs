use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Computers list payload (`GET /computer/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ComputerList {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub busy_executors: Option<u32>,
    #[serde(default)]
    pub total_executors: Option<u32>,
    #[serde(default)]
    pub computer: Vec<ComputerSummary>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// One computer entry in `ComputerList`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ComputerSummary {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub offline: Option<bool>,
    #[serde(default)]
    pub temporarily_offline: Option<bool>,
    #[serde(default)]
    pub idle: Option<bool>,
    #[serde(default)]
    pub num_executors: Option<u32>,
    #[serde(default)]
    pub monitor_data: Option<Value>,
    #[serde(rename = "_class", default)]
    pub class_name: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Computer details payload (`GET /computer/<name>/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ComputerInfo {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub offline: Option<bool>,
    #[serde(default)]
    pub temporarily_offline: Option<bool>,
    #[serde(default)]
    pub idle: Option<bool>,
    #[serde(default)]
    pub num_executors: Option<u32>,
    #[serde(default)]
    pub monitor_data: Option<Value>,
    #[serde(rename = "_class", default)]
    pub class_name: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
