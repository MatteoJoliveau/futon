use crate::{response::ServerInstanceInfo, FutonResult};
use futon_core::{Credentials, Service};

use futon_core::{FutonClient, FutonRequest};
use http::Method;
use url::Url;

pub struct Meta {
    client: FutonClient,
    url: Url,
    credentials: Credentials,
}

impl Meta {
    pub(crate) fn new(client: FutonClient, url: Url, credentials: Credentials) -> Self {
        Self {
            client,
            url,
            credentials,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn is_up(&self) -> FutonResult<bool> {
        let mut client = self.client.clone();
        let req = FutonRequest::new(self.url.clone())?
            .method(Method::HEAD)?
            .credentials(self.credentials.clone())
            .path("_up");

        let res = client.call(req).await?;

        Ok(res.status().is_success())
    }

    #[tracing::instrument(skip(self))]
    pub async fn server_info(&self) -> FutonResult<ServerInstanceInfo> {
        let mut client = self.client.clone();
        let req = FutonRequest::new(self.url.clone())?
            .credentials(self.credentials.clone())
            .path("/");

        let res = client.call(req).await?;

        let info = res.error_for_status()?.body().json()?;
        Ok(info)
    }
}
