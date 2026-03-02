use crate::QueueItemId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Result of triggering a build.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TriggeredBuild {
    /// Queue item id parsed from the `Location` header (when available).
    pub queue_item_id: Option<QueueItemId>,
    /// Raw `Location` header value (when available).
    pub location: Option<Box<str>>,
}

/// Path to an artifact within a build.
///
/// The SDK will translate a path like `a/b.txt` into URL segments: `.../artifact/a/b.txt`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArtifactPath(String);

impl ArtifactPath {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn url_segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/').filter(|segment| !segment.is_empty())
    }
}

impl From<&str> for ArtifactPath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ArtifactPath {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Result of `.../logText/progressiveText`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProgressiveText {
    /// Newly appended console output starting from the requested offset.
    pub text: String,
    /// Next start offset (`X-Text-Size`) for the following request (when present).
    pub next_start: Option<u64>,
    /// Whether the server indicates more data is available (`X-More-Data: true`).
    pub more_data: bool,
}

/// One job entry in `.../api/json` results.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct JobSummary {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Job list payload (for example `GET /api/json?tree=jobs[...]`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JobList {
    #[serde(default)]
    pub jobs: Vec<JobSummary>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Job-level API payload (for example `GET /job/<name>/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct JobInfo {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub buildable: Option<bool>,
    #[serde(default)]
    pub in_queue: Option<bool>,
    #[serde(default)]
    pub next_build_number: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Build-level API payload (for example `GET /job/<name>/<build>/api/json`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuildInfo {
    #[serde(default)]
    pub number: Option<u64>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub building: Option<bool>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub full_display_name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub duration: Option<u64>,
    #[serde(default)]
    pub timestamp: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
