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
    #[error("querystring error: {0}")]
    QueryString(#[from] serde_qs::Error),
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    #[error("{0}")]
    NotFound(ErrorResponse),
    #[error("{0}")]
    Unauthorized(ErrorResponse),
    #[error("{0}")]
    UnknownError(ErrorResponse),
    #[error("{0}")]
    InvalidRevFormat(ErrorResponse),
    #[error("{0}")]
    UnknownBadRequest(ErrorResponse),
    #[error("{0}")]
    Conflict(ErrorResponse),
    #[error("invalid database name: '{0}'. See: https://docs.couchdb.org/en/stable/api/database/common.html#put--db")]
    InvalidDatabaseName(String),
}

impl FutonError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}
