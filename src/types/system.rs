use serde::Deserialize;

/// JSON payload of `GET /crumbIssuer/api/json`.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Crumb {
    #[serde(rename = "crumbRequestField")]
    pub crumb_request_field: String,
    pub crumb: String,
}
