use crate::QueueItemId;
use serde::{Deserialize, Serialize};

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
