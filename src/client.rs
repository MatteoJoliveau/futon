use bytes::Bytes;
use http::{header, response::Parts, HeaderMap, Method, Request, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tower::Service;
use url::Url;

use crate::{auth::Credentials, error::FutonError, response::ErrorResponse, FutonResult};

#[cfg(feature = "hyper")]
pub mod hyper;

pub trait Client:
    Clone + Service<Request<Bytes>, Response = Response<Bytes>, Error = ClientError>
{
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[cfg(feature = "hyper")]
    #[error("hyper error: {0}")]
    Hyper(#[from] ::hyper::Error),
}

pub(crate) async fn head_request<H: Into<HeaderMap>>(
    client: &mut impl Client,
    method: Method,
    url: Url,
    credentials: &Credentials,
    headers: Option<H>,
) -> FutonResult<Parts> {
    let mut builder = Request::builder()
        .method(method)
        .uri(url.to_string().parse::<http::uri::Uri>().unwrap());

    if let Some((map, headers)) = builder.headers_mut().zip(headers) {
        *map = headers.into();
    }

    let mut request = builder.body(Bytes::default())?;

    if let Some((name, value)) = credentials.header() {
        request.headers_mut().append(name, value);
    }

    let res = client.call(request).await?;
    let (parts, _body) = res.into_parts();
    Ok(parts)
}

#[tracing::instrument(skip(client, body))]
pub(crate) async fn json_request<T: Serialize, R: DeserializeOwned>(
    client: &mut impl Client,
    method: Method,
    url: Url,
    credentials: &Credentials,
    body: Option<T>,
) -> FutonResult<R> {
    let has_body = body.is_some();
    let body = match body {
        Some(body) => Bytes::from(serde_json::to_vec(&body)?),
        None => Bytes::default(),
    };

    let mut request = Request::builder()
        .method(method)
        .header(header::ACCEPT, "application/json")
        .uri(url.to_string().parse::<http::uri::Uri>().unwrap())
        .body(body)?;

    if has_body {
        request
            .headers_mut()
            .append(header::CONTENT_TYPE, "application/json".parse().unwrap());
    }

    if let Some((name, value)) = credentials.header() {
        request.headers_mut().append(name, value);
    }

    eprintln!("{request:?}");
    let res = client.call(request).await?;
    eprintln!("{res:?}");
    let res = response_to_error(res)?;

    // optimization to avoid trying to deserialize a JSON response when the user will ignore it anyway
    // size_of::<R>() == 0 usually means an empty tuple
    let res = if std::mem::size_of::<R>() == 0 {
        serde_json::from_value(serde_json::Value::Null)?
    } else {
        serde_json::from_slice(res.body())?
    };
    Ok(res)
}

fn response_to_error(response: Response<Bytes>) -> FutonResult<Response<Bytes>> {
    if response.status().is_success() {
        return Ok(response);
    }

    let error = if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
        match content_type.as_bytes() {
            b"application/json" => serde_json::from_slice(response.body())?,
            content_type => ErrorResponse {
                error: "unsupported content type".to_string(),
                reason: format!("content type '{content_type:?}' is unsupported"),
            },
        }
    } else {
        ErrorResponse {
            error: "unknown error".to_string(),
            reason: "unknown error".to_string(),
        }
    };

    let err = match response.status() {
        StatusCode::NOT_FOUND => FutonError::NotFound(error),
        StatusCode::UNAUTHORIZED => FutonError::Unauthorized(error),
        _ => FutonError::UnknownError(error),
    };

    Err(err)
}
