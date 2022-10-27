use http::{header::HeaderName, HeaderMap, HeaderValue, Method, Request};
use hyper::{body::Bytes, Body};
use serde::Serialize;
use std::{
    borrow::Borrow,
    convert::Infallible,
    fmt::{Debug, Display},
};
use url::Url;

use crate::{Credentials, FutonBody};

#[derive(Debug)]
pub struct FutonRequest {
    pub(crate) url: Url,
    pub(crate) method: Method,
    credentials: Credentials,
    headers: HeaderMap,
    body: FutonBody,
}

impl Display for FutonRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.method, self.url)?;
        match &self.credentials {
            Credentials::Basic(_, _) => write!(f, "\nauthorization: Basic [REDACTED]")?,
            Credentials::None => {}
        }

        for name in self.headers.keys() {
            for value in self.headers.get_all(name) {
                write!(
                    f,
                    "\n{}: {}",
                    name,
                    String::from_utf8_lossy(value.as_bytes())
                )?;
            }
        }

        let bytes = self.body.bytes();
        if !bytes.is_empty() {
            write!(f, "\n\n{}", String::from_utf8_lossy(&bytes))?;
        }

        Ok(())
    }
}

impl FutonRequest {
    pub fn new<U>(url: U) -> Result<Self, RequestError>
    where
        Url: TryFrom<U>,
        <Url as TryFrom<U>>::Error: Into<RequestError>,
    {
        Ok(Self {
            url: TryFrom::try_from(url).map_err(Into::into)?,
            credentials: Credentials::default(),
            headers: HeaderMap::default(),
            method: Method::default(),
            body: FutonBody::default(),
        })
    }

    pub fn method<M>(mut self, method: M) -> Result<Self, RequestError>
    where
        Method: TryFrom<M>,
        <Method as TryFrom<M>>::Error: Into<RequestError>,
    {
        self.method = TryFrom::try_from(method).map_err(Into::into)?;
        Ok(self)
    }

    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = credentials;
        self
    }

    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        self.url.query_pairs_mut().append_pair(key, value);
        self
    }

    pub fn query_params<K, V, I>(mut self, params: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.url.query_pairs_mut().extend_pairs(params);
        self
    }

    pub fn query_string<Q: Serialize>(mut self, query: &Q) -> Result<Self, RequestError> {
        let qs = serde_qs::to_string(query)?;
        self.url.set_query(Some(&qs));
        Ok(self)
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self, RequestError>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<RequestError>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<RequestError>,
    {
        let name = <HeaderName as TryFrom<K>>::try_from(key).map_err(Into::into)?;
        let value = <HeaderValue as TryFrom<V>>::try_from(value).map_err(Into::into)?;
        self.headers.append(name, value);
        Ok(self)
    }

    pub fn body(mut self, body: Bytes) -> Self {
        self.body = body.into();
        self
    }

    pub fn json<B: Serialize + Debug>(mut self, body: B) -> Result<Self, RequestError> {
        self.body = FutonBody::from_json(body)?;
        self.header(http::header::CONTENT_TYPE, "application/json")
    }

    pub fn path(mut self, path: &str) -> Self {
        self.url.set_path(path);
        self
    }

    #[inline]
    pub fn database(self, db: &str) -> Self {
        self.path(db)
    }

    pub fn document(mut self, id: &str, rev: Option<&str>) -> Self {
        if self.url.path() == "/" {
            panic!("cannot construct a document URL without a database prefix",);
        }

        self.url.path_segments_mut().unwrap().push(id);
        if let Some(rev) = rev {
            self.url.query_pairs_mut().append_pair("rev", rev);
        }

        self
    }
}

impl TryFrom<FutonRequest> for Request<Body> {
    type Error = crate::Error;

    fn try_from(request: FutonRequest) -> Result<Self, Self::Error> {
        let body = Body::from(request.body);
        let mut req = Request::builder()
            .method(request.method)
            .uri(&request.url.to_string())
            .body(body)?;

        if let Some((name, value)) = request.credentials.as_header() {
            req.headers_mut().append(name, value);
        }

        req.headers_mut().extend(request.headers);

        Ok(req)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("invalid method name: {0}")]
    Method(#[from] http::method::InvalidMethod),
    #[error("invalid uri: {0}")]
    Uri(String),
    #[error("querystring serialization error: {0}")]
    Query(#[from] serde_qs::Error),
    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("this error cannot exist!")]
    Infallible(#[from] Infallible),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_builds_a_futon_request() -> anyhow::Result<()> {
        let req = FutonRequest::new("https://example.com")?
            .credentials(Credentials::basic("hello", "world"))
            .header("test", "header")?
            .method("PUT")?
            .database("test")?
            .document("example", None)?
            .json("test")?;

        assert_eq!(
            req.url.to_string(),
            "https://example.com/test/example".to_string()
        );
        assert_eq!(req.credentials, Credentials::basic("hello", "world"));
        assert_eq!(req.headers.get("test").unwrap(), &"header");
        assert_eq!(req.method, Method::PUT);
        assert_eq!(req.body.inner.as_ref(), b"\"test\"");

        let req: Request<Body> = req.try_into()?;
        assert_eq!(req.uri(), "https://example.com/test/example");
        assert_eq!(req.method(), Method::PUT);
        assert_eq!(
            req.headers().get(http::header::AUTHORIZATION).unwrap(),
            &"Basic aGVsbG86d29ybGQ="
        );
        assert_eq!(req.headers().get("test").unwrap(), &"header");

        Ok(())
    }
}
