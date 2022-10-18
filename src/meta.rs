use crate::{
    auth::Credentials,
    client::{head_request, json_request, Client},
    response::ServerInstanceInfo,
    FutonResult,
};

use http::{HeaderMap, Method};
use url::Url;

pub struct Meta<C: Client> {
    client: C,
    url: Url,
    credentials: Credentials,
}

impl<C: Client> Meta<C> {
    pub(crate) fn new(client: C, url: Url, credentials: Credentials) -> Self {
        Self {
            client,
            url,
            credentials,
        }
    }

    pub async fn is_up(&self) -> FutonResult<bool> {
        let mut client = self.client.clone();
        let parts = head_request::<HeaderMap>(
            &mut client,
            Method::GET,
            self.url.join("/_up").unwrap(),
            &self.credentials,
            None,
        )
        .await?;
        Ok(parts.status.is_success())
    }

    pub async fn server_info(&self) -> FutonResult<ServerInstanceInfo> {
        let mut client = self.client.clone();
        json_request::<(), ServerInstanceInfo>(
            &mut client,
            Method::GET,
            self.url.join("/").unwrap(),
            &self.credentials,
            None,
        )
        .await
    }
}
