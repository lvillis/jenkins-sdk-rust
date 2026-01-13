use crate::{Auth, BodySnippetConfig};
use http::HeaderMap;

use super::redact::{redact_text, truncate_utf8};

pub(crate) fn request_id(headers: &HeaderMap) -> Option<Box<str>> {
    for name in [
        "x-request-id",
        "x-correlation-id",
        "x-amzn-requestid",
        "x-amz-request-id",
    ] {
        if let Some(value) = headers.get(name).and_then(|v| v.to_str().ok()) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string().into_boxed_str());
            }
        }
    }
    None
}

pub(crate) fn extract_message(body: &[u8]) -> Option<Box<str>> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return None;
    };

    let candidates = ["message", "error", "error_message", "Message", "Error"];
    for key in candidates {
        if let Some(msg) = value.get(key).and_then(|v| v.as_str()) {
            let msg = msg.trim();
            if !msg.is_empty() {
                return Some(msg.to_string().into_boxed_str());
            }
        }
    }
    None
}

pub(crate) fn body_snippet(
    body: &[u8],
    config: BodySnippetConfig,
    auth: Option<&Auth>,
) -> Option<Box<str>> {
    if !config.enabled {
        return None;
    }

    let body = String::from_utf8_lossy(body);
    let snippet = truncate_utf8(&body, config.max_bytes).to_string();
    Some(redact_text(snippet, auth).into_boxed_str())
}
