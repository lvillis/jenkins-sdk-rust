//! Type-safe endpoint definitions.

use http::Method;
use serde::de::DeserializeOwned;
use std::borrow::Cow;

/// Common trait implemented by every Jenkins API endpoint.
pub trait Endpoint {
    type Output: DeserializeOwned + Send + 'static;
    fn method(&self) -> Method;
    fn path(&self) -> Cow<'static, str>;
    fn params(&self) -> Option<Vec<(Cow<'static, str>, Cow<'static, str>)>> {
        None
    }
}

use serde_json::Value;

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
        Cow::Borrowed("api/json?tree=jobs[name,url,color]")
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
                    (
                        Cow::Owned(k.clone()),
                        Cow::Owned(v.as_str().unwrap_or("").to_owned()),
                    )
                })
                .collect()
        })
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
}
