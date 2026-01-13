//! Data structures returned by Jenkins APIs.

use serde::{Deserialize, Serialize};

/// A Jenkins job name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobName(String);

impl JobName {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for JobName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for JobName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A Jenkins job path (supports nested items like `folder/job`).
///
/// The SDK will translate a path like `a/b` into URL segments: `/job/a/job/b/...`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobPath(String);

impl JobPath {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn url_segments(&self) -> impl Iterator<Item = &str> {
        self.0
            .split('/')
            .filter(|segment| !segment.is_empty())
            .flat_map(|segment| ["job", segment])
    }
}

impl From<JobName> for JobPath {
    fn from(value: JobName) -> Self {
        Self(value.0)
    }
}

impl From<&str> for JobPath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for JobPath {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A Jenkins build number (treated as a string for maximum compatibility).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuildNumber(String);

impl BuildNumber {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for BuildNumber {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for BuildNumber {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A Jenkins queue item id (treated as a string for maximum compatibility).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueueItemId(String);

impl QueueItemId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for QueueItemId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for QueueItemId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A Jenkins view name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewName(String);

impl ViewName {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ViewName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ViewName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A Jenkins user id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(String);

impl UserId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for UserId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A Jenkins computer/node name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComputerName(String);

impl ComputerName {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ComputerName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ComputerName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A Jenkins job entry in `GET /api/json`.
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Job {
    pub name: String,
    pub url: String,
    pub color: String,
}

/// Executor statistics from `GET /computer/api/json`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExecutorsInfo {
    pub total_executors: u32,
    pub busy_executors: u32,
    #[serde(skip)]
    pub idle_executors: u32,
}

impl ExecutorsInfo {
    /// Populate `idle_executors`.
    pub fn calc_idle(mut self) -> Self {
        self.idle_executors = self.total_executors - self.busy_executors;
        self
    }
}
