use thiserror::Error;

use crate::{client::ClientError, response::ErrorResponse};

#[derive(Debug, Error)]
pub enum FutonError {
    #[error("http error: {0}")]
    Http(#[from] http::Error),
    #[error("client error: {0}")]
    Client(#[from] ClientError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    #[error("{0}")]
    NotFound(ErrorResponse),
    #[error("{0}")]
    Unauthorized(ErrorResponse),
    #[error("{0}")]
    UnknownError(ErrorResponse),
}

impl FutonError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}
