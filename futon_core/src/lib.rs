mod body;
mod credentials;
mod error;
mod request;
mod response;

use std::{future::Future, pin::Pin};

pub use body::FutonBody;
pub use credentials::Credentials;
pub use error::Error;
use http::Response;
use hyper::client::HttpConnector;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
pub use request::{FutonRequest, RequestError};
pub use response::{ErrorResponse, FutonResponse};
pub use tower::Service;
use tracing::Instrument;

#[derive(Clone)]
pub struct FutonClient {
    inner: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Default for FutonClient {
    fn default() -> Self {
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        Self {
            inner: hyper::Client::builder().build(https),
        }
    }
}

impl Service<FutonRequest> for FutonClient {
    type Response = FutonResponse;

    type Error = error::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    #[tracing::instrument(skip(self, req), fields(url=%req.url, method=%req.method))]
    fn call(&mut self, req: FutonRequest) -> Self::Future {
        let inner = self.inner.clone();
        let mut client = std::mem::replace(&mut self.inner, inner);
        Box::pin(
            async move {
                let res = client.call(req.try_into()?).await?;
                let (parts, body) = res.into_parts();
                let body = hyper::body::to_bytes(body).await?;
                let res = Response::from_parts(parts, body);
                let res = FutonResponse::try_from(res)?;
                tracing::debug!(status = %res.status(), ?res, "request completed");
                Ok(res)
            }
            .instrument(tracing::debug_span!("request")),
        )
    }
}
