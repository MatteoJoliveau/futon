use http::{
    header::{self, HeaderName},
    HeaderValue,
};
use secstr::SecUtf8;

#[derive(Debug, Clone)]
pub enum Credentials {
    Basic { username: String, password: SecUtf8 },
    Empty,
}

impl Default for Credentials {
    fn default() -> Self {
        Self::Empty
    }
}

impl Credentials {
    pub fn basic(username: impl ToString, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.to_string(),
            password: SecUtf8::from(password),
        }
    }

    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn header(&self) -> Option<(HeaderName, HeaderValue)> {
        match self {
            Credentials::Basic { username, password } => {
                let b64 = base64::encode(format!("{username}:{}", password.unsecure()));
                Some((
                    header::AUTHORIZATION,
                    format!("Basic {b64}").parse().unwrap(),
                ))
            }
            Credentials::Empty => None,
        }
    }
}
