use http::Method;
use url::Url;

use crate::{
    auth::Credentials,
    client::{json_request, Client},
    response::DatabaseInfo,
    FutonResult,
};

pub struct Database<C: Client> {
    client: C,
    url: Url,
    credentials: Credentials,
}

impl<C: Client> Database<C> {
    pub(crate) fn new(client: C, url: Url, credentials: Credentials) -> Self {
        Self {
            client,
            url,
            credentials,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn info(&self) -> FutonResult<DatabaseInfo> {
        let mut client = self.client.clone();
        json_request::<(), DatabaseInfo>(
            &mut client,
            Method::GET,
            self.url.clone(),
            &self.credentials,
            None,
        )
        .await
    }

    pub async fn create(&self) -> FutonResult<()> {
        let mut client = self.client.clone();
        json_request::<(), ()>(
            &mut client,
            Method::PUT,
            self.url.clone(),
            &self.credentials,
            None,
        )
        .await
    }

    pub async fn delete(&self) -> FutonResult<()> {
        let mut client = self.client.clone();
        json_request::<(), ()>(
            &mut client,
            Method::DELETE,
            self.url.clone(),
            &self.credentials,
            None,
        )
        .await
    }
}
