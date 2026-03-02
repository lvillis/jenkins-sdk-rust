use crate::Error;
use http::Uri;
use std::env;
use url::Url;

#[derive(Debug, Default)]
pub(crate) struct ProxyEnvConfig {
    pub(crate) proxy_uri: Option<Uri>,
    pub(crate) no_proxy_rules: Vec<String>,
}

/// Resolve proxy/no_proxy settings from process environment.
///
/// `disable_system_proxy=true` skips reading any proxy env variables.
pub(crate) fn load_proxy_env(
    base_url: &str,
    disable_system_proxy: bool,
) -> Result<ProxyEnvConfig, Error> {
    load_proxy_env_with_lookup(base_url, disable_system_proxy, |key| env::var(key).ok())
}

fn load_proxy_env_with_lookup(
    base_url: &str,
    disable_system_proxy: bool,
    mut lookup: impl FnMut(&str) -> Option<String>,
) -> Result<ProxyEnvConfig, Error> {
    if disable_system_proxy {
        return Ok(ProxyEnvConfig::default());
    }

    let scheme = Url::parse(base_url).ok().map(|url| url.scheme().to_owned());

    let proxy_uri = match proxy_env_value(scheme.as_deref(), &mut lookup) {
        Some((key, value)) => {
            let uri = value
                .parse::<Uri>()
                .map_err(|source| Error::InvalidConfig {
                    message: format!("invalid proxy URI in environment variable `{key}`").into(),
                    source: Some(Box::new(source)),
                })?;
            Some(uri)
        }
        None => None,
    };

    let no_proxy_rules = match first_non_empty_env(&["NO_PROXY", "no_proxy"], &mut lookup) {
        Some((_, value)) => value
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        None => Vec::new(),
    };

    Ok(ProxyEnvConfig {
        proxy_uri,
        no_proxy_rules,
    })
}

fn proxy_env_value(
    scheme: Option<&str>,
    lookup: &mut impl FnMut(&str) -> Option<String>,
) -> Option<(&'static str, String)> {
    match scheme {
        Some("https") => first_non_empty_env(
            &[
                "HTTPS_PROXY",
                "https_proxy",
                "ALL_PROXY",
                "all_proxy",
                "HTTP_PROXY",
                "http_proxy",
            ],
            lookup,
        ),
        Some("http") => first_non_empty_env(
            &["HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"],
            lookup,
        ),
        _ => first_non_empty_env(
            &[
                "HTTPS_PROXY",
                "https_proxy",
                "HTTP_PROXY",
                "http_proxy",
                "ALL_PROXY",
                "all_proxy",
            ],
            lookup,
        ),
    }
}

fn first_non_empty_env(
    keys: &[&'static str],
    lookup: &mut impl FnMut(&str) -> Option<String>,
) -> Option<(&'static str, String)> {
    keys.iter().find_map(|&key| {
        let value = lookup(key)?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some((key, trimmed.to_owned()))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::load_proxy_env_with_lookup;
    use crate::Error;
    use std::collections::BTreeMap;

    #[test]
    fn https_prefers_https_proxy() {
        let env = env_map([("HTTP_PROXY", "http://h"), ("HTTPS_PROXY", "http://s")]);
        let cfg = load_proxy_env_with_lookup("https://jenkins.example.com", false, lookup(&env))
            .expect("proxy env should parse");
        assert_eq!(
            cfg.proxy_uri.as_ref().map(ToString::to_string).as_deref(),
            Some("http://s/")
        );
    }

    #[test]
    fn https_falls_back_to_all_proxy() {
        let env = env_map([("ALL_PROXY", "http://all:8080")]);
        let cfg = load_proxy_env_with_lookup("https://jenkins.example.com", false, lookup(&env))
            .expect("proxy env should parse");
        assert_eq!(
            cfg.proxy_uri.as_ref().map(ToString::to_string).as_deref(),
            Some("http://all:8080/")
        );
    }

    #[test]
    fn http_prefers_http_proxy_over_all_proxy() {
        let env = env_map([
            ("HTTP_PROXY", "http://http-only:8080"),
            ("ALL_PROXY", "http://all:8080"),
        ]);
        let cfg = load_proxy_env_with_lookup("http://jenkins.example.com", false, lookup(&env))
            .expect("proxy env should parse");
        assert_eq!(
            cfg.proxy_uri.as_ref().map(ToString::to_string).as_deref(),
            Some("http://http-only:8080/")
        );
    }

    #[test]
    fn no_proxy_rules_are_split_and_trimmed() {
        let env = env_map([
            ("HTTPS_PROXY", "http://proxy:8080"),
            ("NO_PROXY", " localhost, .example.com ,10.0.0.0/8, "),
        ]);
        let cfg = load_proxy_env_with_lookup("https://jenkins.example.com", false, lookup(&env))
            .expect("proxy env should parse");
        assert_eq!(
            cfg.no_proxy_rules,
            vec![
                "localhost".to_owned(),
                ".example.com".to_owned(),
                "10.0.0.0/8".to_owned()
            ]
        );
    }

    #[test]
    fn disable_system_proxy_ignores_all_env_vars() {
        let env = env_map([
            ("HTTPS_PROXY", "http://proxy:8080"),
            ("NO_PROXY", "localhost"),
        ]);
        let cfg = load_proxy_env_with_lookup("https://jenkins.example.com", true, lookup(&env))
            .expect("proxy env should parse");
        assert!(cfg.proxy_uri.is_none());
        assert!(cfg.no_proxy_rules.is_empty());
    }

    #[test]
    fn invalid_proxy_uri_returns_invalid_config() {
        let env = env_map([("HTTPS_PROXY", "http://bad host")]);
        let err = load_proxy_env_with_lookup("https://jenkins.example.com", false, lookup(&env))
            .expect_err("invalid URI should fail");
        match err {
            Error::InvalidConfig { message, .. } => {
                assert!(message.contains("HTTPS_PROXY"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    fn env_map<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<String, String> {
        pairs
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }

    fn lookup<'a>(env: &'a BTreeMap<String, String>) -> impl FnMut(&str) -> Option<String> + 'a {
        move |key| env.get(key).cloned()
    }
}
