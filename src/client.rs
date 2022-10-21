use bytes::Bytes;
use http::{header, response::Parts, HeaderMap, Method, Request, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
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

impl<S> Client for S where
    S: Clone + Service<Request<Bytes>, Response = Response<Bytes>, Error = ClientError>
{
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[cfg(feature = "hyper")]
    #[error("hyper error: {0}")]
    Hyper(#[from] ::hyper::Error),
}

#[tracing::instrument(skip(client, url), fields(url = %url))]
pub(crate) async fn head_request<H: Debug + Into<HeaderMap>>(
    client: &mut impl Client,
    url: Url,
    credentials: &Credentials,
    headers: Option<H>,
) -> FutonResult<Parts> {
    let mut builder = Request::builder()
        .method(Method::HEAD)
        .uri(&url.to_string());

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

#[tracing::instrument(skip(client, request))]
pub(crate) async fn json_request<R: Serialize + Debug, T: DeserializeOwned>(
    client: &mut impl Client,
    credentials: &Credentials,
    request: Request<R>,
) -> FutonResult<T> {
    let (mut parts, body) = request.into_parts();
    let has_body = std::mem::size_of_val(&body) == 0;

    let body = if has_body {
        Bytes::from(serde_json::to_vec(&body)?)
    } else {
        Bytes::new()
    };

    parts
        .headers
        .insert(header::ACCEPT, "application/json".parse().unwrap());
    parts
        .headers
        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    if let Some((name, value)) = credentials.header() {
        parts.headers.append(name, value);
    }

    let request = Request::from_parts(parts, body);

    let res = client.call(request).await?;
    let res = response_to_error(res)?;

    // optimization to avoid trying to deserialize a JSON response when the user will ignore it anyway
    // size_of::<T>() == 0 usually means an empty tuple
    let res = if std::mem::size_of::<T>() == 0 {
        serde_json::from_value(serde_json::Value::Null)?
    } else {
        serde_json::from_slice(res.body())?
    };
    Ok(res)
}

#[tracing::instrument(skip(client, url, body), fields(url = %url))]
pub(crate) async fn old_json_request<T: Serialize + std::fmt::Debug, R: DeserializeOwned>(
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

    let res = client.call(request).await?;
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

#[tracing::instrument(skip(client, url, body), fields(url = %url))]
pub(crate) async fn maybe_json_request<T: Serialize + std::fmt::Debug, R: DeserializeOwned>(
    client: &mut impl Client,
    method: Method,
    url: Url,
    credentials: &Credentials,
    body: Option<T>,
) -> FutonResult<Option<R>> {
    match old_json_request(client, method, url, credentials, body).await {
        Ok(res) => Ok(Some(res)),
        Err(err) if err.is_not_found() => Ok(None),
        Err(err) => Err(err),
    }
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
        StatusCode::CONFLICT => FutonError::Conflict(error),
        StatusCode::BAD_REQUEST => match error.reason.to_lowercase().trim() {
            "invalid rev format" => FutonError::InvalidRevFormat(error),
            _ => FutonError::UnknownBadRequest(error),
        },
        _ => FutonError::UnknownError(error),
    };

    Err(err)
}
