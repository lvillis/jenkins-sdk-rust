use crate::Auth;

pub(crate) fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

pub(crate) fn redact_text(mut text: String, auth: Option<&Auth>) -> String {
    let Some(auth) = auth else {
        return text;
    };

    for secret in auth.secrets() {
        if !secret.is_empty() {
            text = text.replace(secret, "<redacted>");
        }
    }
    text
}
