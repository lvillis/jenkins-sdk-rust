//! Data structures returned by Jenkins APIs.

use serde::{Deserialize, Serialize};

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
