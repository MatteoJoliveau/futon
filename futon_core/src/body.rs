use std::string::FromUtf8Error;

use hyper::{body::Bytes, Body};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct FutonBody {
    inner: Bytes,
}

impl Default for FutonBody {
    fn default() -> Self {
        Self::empty()
    }
}

impl FutonBody {
    pub fn empty() -> Self {
        Self {
            inner: Bytes::default(),
        }
    }

    pub fn from_json<T: Serialize>(body: T) -> Result<Self, serde_json::Error> {
        let bytes = serde_json::to_vec(&body)?;
        Ok(Self {
            inner: bytes.into(),
        })
    }

    pub fn bytes(&self) -> Bytes {
        self.inner.clone()
    }

    pub fn text(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.inner.to_vec())
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        let body = serde_json::from_slice(&self.inner)?;
        Ok(body)
    }
}

impl AsRef<[u8]> for FutonBody {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<FutonBody> for Body {
    fn from(body: FutonBody) -> Self {
        Body::from(body.inner)
    }
}

impl From<Bytes> for FutonBody {
    fn from(inner: Bytes) -> Self {
        Self { inner }
    }
}
