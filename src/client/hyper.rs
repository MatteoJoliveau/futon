use std::{future::Future, pin::Pin};

use bytes::Bytes;
use hyper::{client::HttpConnector, Body, Client, Request, Response};
use tower::Service;
use tracing::Instrument;

use super::ClientError;

type Hyper = Client<hyper_rustls::HttpsConnector<HttpConnector>>;

#[derive(Clone)]
pub struct HyperClient {
    client: Hyper,
}

impl HyperClient {
    pub fn new() -> Self {
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        let client = Client::builder().build(https);

        Self { client }
    }
}

impl Default for HyperClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Service<Request<Bytes>> for HyperClient {
    type Response = Response<Bytes>;

    type Error = ClientError;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.client.poll_ready(cx).map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    fn call(&mut self, req: Request<Bytes>) -> Self::Future {
        let client = self.client.clone();
        let mut inner = std::mem::replace(&mut self.client, client);
        let span = tracing::Span::current();
        Box::pin(
            async move {
                let (parts, payload) = req.into_parts();
                let body = Body::from(payload);
                let req = Request::from_parts(parts, body);

                let res = inner
                    .call(req)
                    .instrument(tracing::trace_span!("hyper request"))
                    .await?;

                let (parts, body) = res.into_parts();
                let bytes = hyper::body::to_bytes(body).await?;
                let res = Response::from_parts(parts, bytes);
                tracing::trace!(?res, "http call");
                Ok(res)
            }
            .instrument(span),
        )
    }
}
