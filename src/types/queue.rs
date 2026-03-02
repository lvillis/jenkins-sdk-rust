use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Queue list payload (`GET /queue/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QueueList {
    #[serde(default)]
    pub items: Vec<QueueItem>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Queue item payload (`GET /queue/item/<id>/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueueItem {
    #[serde(default, deserialize_with = "deserialize_opt_string_or_number")]
    pub id: Option<String>,
    #[serde(default)]
    pub blocked: Option<bool>,
    #[serde(default)]
    pub buildable: Option<bool>,
    #[serde(default)]
    pub stuck: Option<bool>,
    #[serde(default)]
    pub cancelled: Option<bool>,
    #[serde(default)]
    pub why: Option<String>,
    #[serde(default)]
    pub params: Option<String>,
    #[serde(default)]
    pub in_queue_since: Option<u64>,
    #[serde(default)]
    pub task: Option<QueueTask>,
    #[serde(default)]
    pub executable: Option<QueueExecutable>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Minimal task shape nested in queue items.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueueTask {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(rename = "_class", default)]
    pub class_name: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Build pointer nested in queue items when executable already started.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueueExecutable {
    #[serde(default, deserialize_with = "deserialize_opt_string_or_number")]
    pub number: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

fn deserialize_opt_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(other) => Err(D::Error::custom(format!(
            "expected string/number/null, got {other}"
        ))),
    }
}
