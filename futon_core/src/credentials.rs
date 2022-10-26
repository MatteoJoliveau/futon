use http::{header::HeaderName, HeaderValue};
use secstr::SecUtf8;

#[derive(Clone, Debug, PartialEq)]
pub enum Credentials {
    Basic(String, SecUtf8),
    None,
}

impl Credentials {
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic(username.into(), SecUtf8::from(password))
    }

    pub fn none() -> Self {
        Self::None
    }

    pub(crate) fn as_header(&self) -> Option<(HeaderName, HeaderValue)> {
        match self {
            Credentials::Basic(username, password) => {
                let value = base64::encode(format!("{username}:{}", password.unsecure()));
                Some((
                    http::header::AUTHORIZATION,
                    format!("Basic {}", value).parse::<HeaderValue>().unwrap(),
                ))
            }
            Credentials::None => None,
        }
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self::None
    }
}
