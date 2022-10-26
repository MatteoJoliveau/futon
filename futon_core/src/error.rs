use crate::{request::RequestError, response::ErrorResponse};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http protocol error: {0}")]
    Http(#[from] http::Error),
    #[error("http transport error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to construct HTTP request: {0}")]
    RequestError(#[from] RequestError),
    #[error("{0}")]
    CouchError(#[from] ErrorResponse),
}
