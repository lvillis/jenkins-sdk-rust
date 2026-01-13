use crate::Error;
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
use std::fmt;

#[derive(Clone, Default, Eq, PartialEq)]
pub struct SecretString(String);

impl SecretString {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Auth {
    Basic { user: String, token: SecretString },
    Bearer { token: SecretString },
}

impl Auth {
    #[must_use]
    pub fn basic(user: impl Into<String>, token: impl Into<String>) -> Self {
        Self::Basic {
            user: user.into(),
            token: SecretString::new(token),
        }
    }

    #[must_use]
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer {
            token: SecretString::new(token),
        }
    }

    pub(crate) fn secrets(&self) -> Vec<&str> {
        match self {
            Self::Basic { token, .. } => vec![token.expose()],
            Self::Bearer { token } => vec![token.expose()],
        }
    }

    pub(crate) fn apply(&self, headers: &mut HeaderMap) -> Result<(), Error> {
        let value = match self {
            Self::Basic { user, token } => {
                let raw = format!("Basic {}", B64.encode(format!("{user}:{}", token.expose())));
                HeaderValue::from_str(&raw).map_err(|err| Error::InvalidConfig {
                    message: "invalid Authorization header value".into(),
                    source: Some(Box::new(err)),
                })?
            }
            Self::Bearer { token } => {
                let raw = format!("Bearer {}", token.expose());
                HeaderValue::from_str(&raw).map_err(|err| Error::InvalidConfig {
                    message: "invalid Authorization header value".into(),
                    source: Some(Box::new(err)),
                })?
            }
        };

        headers.insert(AUTHORIZATION, value);
        Ok(())
    }
}
