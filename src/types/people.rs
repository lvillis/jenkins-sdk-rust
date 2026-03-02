use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// People list payload (`GET /people/api/json` or `/asynchPeople/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PeopleList {
    #[serde(default)]
    pub users: Vec<PersonEntry>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// One person entry in `PeopleList`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PersonEntry {
    #[serde(default)]
    pub user: Option<PersonUser>,
    #[serde(default)]
    pub project: Option<PersonProject>,
    #[serde(default)]
    pub last_change: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Minimal nested user shape in people results.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PersonUser {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub absolute_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Minimal nested project shape in people results.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PersonProject {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
