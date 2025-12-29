//! Type-safe endpoint definitions.

use crate::core::JenkinsError;
use http::Method;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::borrow::Cow;

/// Common trait implemented by every Jenkins API endpoint.
pub trait Endpoint {
    type Output: DeserializeOwned + Send + 'static;

    fn method(&self) -> Method;
    /// Relative API path, without a leading `/`.
    ///
    /// Keep the path free of query strings; use [`Endpoint::params`] for query/form fields.
    fn path(&self) -> Cow<'static, str>;
    /// Query/form parameters.
    ///
    /// Parameters are encoded as query string for `GET`, otherwise as form fields.
    fn params(&self) -> Option<Vec<(Cow<'static, str>, Cow<'static, str>)>> {
        None
    }
    fn parse(&self, body: String) -> Result<Self::Output, JenkinsError> {
        Ok(serde_json::from_str(&body)?)
    }
}

/// GET /queue/api/json
pub struct QueueLength;
impl Endpoint for QueueLength {
    type Output = Value;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("queue/api/json")
    }
}

/// GET /computer/api/json
pub struct ExecutorsInfoEndpoint;
impl Endpoint for ExecutorsInfoEndpoint {
    type Output = crate::core::ExecutorsInfo;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("computer/api/json")
    }
}

/// GET /api/json?tree=jobs[name,url,color]
pub struct JobsInfo;
impl Endpoint for JobsInfo {
    type Output = Value;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("api/json")
    }
    fn params(&self) -> Option<Vec<(Cow<'static, str>, Cow<'static, str>)>> {
        Some(vec![(
            Cow::Borrowed("tree"),
            Cow::Borrowed("jobs[name,url,color]"),
        )])
    }
}

/// GET /job/<name>/api/json
pub struct JobDetail<'a>(pub &'a str);
impl<'a> Endpoint for JobDetail<'a> {
    type Output = Value;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(format!("job/{}/api/json", self.0))
    }
}

/// GET /job/<name>/lastBuild/api/json
pub struct LastBuildInfo<'a>(pub &'a str);
impl<'a> Endpoint for LastBuildInfo<'a> {
    type Output = Value;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(format!("job/{}/lastBuild/api/json", self.0))
    }
}

/// GET /job/<name>/lastBuild/consoleText
pub struct LastBuildConsole<'a>(pub &'a str);
impl<'a> Endpoint for LastBuildConsole<'a> {
    type Output = String;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(format!("job/{}/lastBuild/consoleText", self.0))
    }
    fn parse(&self, body: String) -> Result<Self::Output, JenkinsError> {
        Ok(body)
    }
}

/// GET /job/<name>/<build>/consoleText
pub struct ConsoleText<'a>(pub &'a str, pub &'a str);
impl<'a> Endpoint for ConsoleText<'a> {
    type Output = String;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(format!("job/{}/{}/consoleText", self.0, self.1))
    }
    fn parse(&self, body: String) -> Result<Self::Output, JenkinsError> {
        Ok(body)
    }
}

/// POST /job/<name>/buildWithParameters
pub struct TriggerBuild<'a> {
    pub job: &'a str,
    pub params: &'a Value,
}
impl<'a> Endpoint for TriggerBuild<'a> {
    type Output = String;
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(format!("job/{}/buildWithParameters", self.job))
    }
    fn params(&self) -> Option<Vec<(Cow<'static, str>, Cow<'static, str>)>> {
        self.params.as_object().map(|m| {
            m.iter()
                .map(|(k, v)| {
                    let value = v
                        .as_str()
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| v.to_string());
                    (Cow::Owned(k.clone()), Cow::Owned(value))
                })
                .collect()
        })
    }
    fn parse(&self, body: String) -> Result<Self::Output, JenkinsError> {
        Ok(body)
    }
}

/// POST /job/<name>/<build>/stop
pub struct StopBuild<'a> {
    /// Job name.
    pub job: &'a str,
    /// Build number.
    pub build: &'a str,
}

impl<'a> Endpoint for StopBuild<'a> {
    type Output = String;

    fn method(&self) -> Method {
        Method::POST
    }

    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(format!("job/{}/{}/stop", self.job, self.build))
    }
    fn parse(&self, body: String) -> Result<Self::Output, JenkinsError> {
        Ok(body)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn trigger_build_params_convert_non_string_values() {
        let payload = json!({
            "number": 42,
            "flag": true,
            "text": "alpha",
        });
        let endpoint = TriggerBuild {
            job: "core",
            params: &payload,
        };

        let params = endpoint.params().unwrap();
        let mut collected = BTreeMap::new();
        for (k, v) in params {
            collected.insert(k.into_owned(), v.into_owned());
        }

        assert_eq!(collected.get("number"), Some(&"42".to_string()));
        assert_eq!(collected.get("flag"), Some(&"true".to_string()));
        assert_eq!(collected.get("text"), Some(&"alpha".to_string()));
    }

    #[test]
    fn console_text_parse_returns_body() {
        let endpoint = ConsoleText("core", "1");
        let body = String::from("logs");
        let parsed = endpoint.parse(body.clone()).unwrap();
        assert_eq!(parsed, body);
    }
}
