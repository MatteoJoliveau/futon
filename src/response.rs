use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct ServerInstanceInfo {
    pub couchdb: String,
    pub uuid: String,
    pub git_sha: String,
    pub version: String,
    pub vendor: ServerInstanceInfoVendor,
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerInstanceInfoVendor {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseInfo {
    pub db_name: String,
}

#[derive(Debug, Deserialize, Error)]
#[error("{error}: {reason}")]
pub struct ErrorResponse {
    pub error: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct Ok {
    pub ok: bool,
}
