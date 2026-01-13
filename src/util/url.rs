use crate::Error;
use url::Url;

pub(crate) fn normalize_base_url(raw: &str) -> Result<Url, Error> {
    let mut url = Url::parse(raw).map_err(|err| Error::InvalidConfig {
        message: "invalid base_url".into(),
        source: Some(Box::new(err)),
    })?;

    if url.query().is_some() || url.fragment().is_some() {
        return Err(Error::InvalidConfig {
            message: "base_url must not include query or fragment".into(),
            source: None,
        });
    }

    let path = url.path();
    if path != "/" && !path.ends_with('/') {
        url.set_path(&format!("{path}/"));
    }
    Ok(url)
}

pub(crate) fn endpoint_url<'a, I>(base_url: &Url, segments: I) -> Result<Url, Error>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut url = base_url.clone();
    {
        let mut path = url.path_segments_mut().map_err(|_| Error::InvalidConfig {
            message: "base_url must be a hierarchical URL".into(),
            source: None,
        })?;
        path.pop_if_empty();
        for seg in segments {
            path.push(seg);
        }
    }
    Ok(url)
}

pub(crate) fn sanitize_url_for_error(url: &Url) -> Url {
    let mut safe = url.clone();
    safe.set_query(None);
    safe.set_fragment(None);
    let _ = safe.set_username("");
    let _ = safe.set_password(None);
    safe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_url_encodes_path_segments() {
        let base = normalize_base_url("https://example.com/jenkins").unwrap();
        let url = endpoint_url(&base, ["job", "a/b c", "api", "json"]).unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.com/jenkins/job/a%2Fb%20c/api/json"
        );
    }

    #[test]
    fn sanitize_url_for_error_strips_query_fragment_and_userinfo() {
        let url = Url::parse("https://user:pass@example.com/x?y=1#z").unwrap();
        let safe = sanitize_url_for_error(&url);
        assert_eq!(safe.as_str(), "https://example.com/x");
    }
}
