use crate::core::error::JenkinsError;
use url::Url;

pub(crate) fn normalize_base_url(raw: &str) -> Result<Url, JenkinsError> {
    let mut url = Url::parse(raw)?;
    let path = url.path();
    if path != "/" && !path.ends_with('/') {
        url.set_path(&format!("{path}/"));
    }
    Ok(url)
}
