use http::{Response, StatusCode};
use hyper::body::Bytes;
use serde::Deserialize;

use crate::FutonBody;

pub struct FutonResponse {
    status: StatusCode,
    body: FutonBody,
}

impl FutonResponse {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn body(&self) -> &FutonBody {
        &self.body
    }

    pub fn into_body(self) -> FutonBody {
        self.body
    }

    pub fn maybe_body(&self) -> Option<&FutonBody> {
        if self.status == StatusCode::NOT_FOUND {
            return None;
        }

        Some(&self.body)
    }

    pub fn error_for_status(self) -> Result<Self, ErrorResponse> {
        let status = self.status;
        if status.is_client_error() || status.is_server_error() {
            let body = self.into_body();
            let err = match body.json::<ErrorResponse>() {
                Ok(mut err) => {
                    err.status = status;
                    err
                }
                Err(_) => ErrorResponse {
                    status,
                    error: status.canonical_reason().unwrap_or("unknown error").into(),
                    reason: String::from_utf8_lossy(&body.bytes()).into(),
                },
            };

            return Err(err);
        }

        Ok(self)
    }
}

impl TryFrom<Response<Bytes>> for FutonResponse {
    type Error = crate::Error;

    fn try_from(res: Response<Bytes>) -> Result<Self, Self::Error> {
        let (parts, body) = res.into_parts();
        let body = FutonBody::from(body);
        Ok(Self {
            status: parts.status,
            body,
        })
    }
}

#[derive(Debug, Deserialize, thiserror::Error)]
#[error("{error}: {reason}")]
pub struct ErrorResponse {
    pub error: String,
    pub reason: String,
    #[serde(skip, default)]
    pub status: StatusCode,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_builds_a_futon_response() -> anyhow::Result<()> {
        let res = Response::builder()
            .status(200)
            .body(Bytes::from_static(b"\"hello futon\""))?;

        let res = FutonResponse::try_from(res)?;

        assert_eq!(res.status, StatusCode::OK);
        let body = res.body.json::<String>()?;
        assert_eq!(body.as_str(), "hello futon");

        Ok(())
    }
}
