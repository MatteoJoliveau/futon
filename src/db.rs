use http::{HeaderMap, Method, StatusCode};
use url::Url;

use crate::{
    auth::Credentials,
    client::{head_request, json_request, Client},
    document::Documents,
    error::FutonError,
    request::DatabaseCreationParams,
    response::DatabaseInfo,
    FutonResult,
};

const NAME_REGEX: &str = r#"^[a-z][a-z0-9_$()+/-]*$"#;

pub struct Database<C> {
    client: C,
    url: Url,
    credentials: Credentials,
}

impl<C: Client> Database<C> {
    pub(crate) fn new(
        client: C,
        url: Url,
        name: &str,
        credentials: Credentials,
    ) -> FutonResult<Self> {
        let re = regex::Regex::new(NAME_REGEX).unwrap();
        if !re.is_match(name) {
            return Err(FutonError::InvalidDatabaseName(name.to_string()));
        }

        Ok(Self {
            client,
            url: url.join(name)?,
            credentials,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn exists(&self) -> FutonResult<bool> {
        let mut client = self.client.clone();
        let parts =
            head_request::<HeaderMap>(&mut client, self.url.clone(), &self.credentials, None)
                .await?;
        Ok(parts.status != StatusCode::NOT_FOUND)
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

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, params: DatabaseCreationParams) -> FutonResult<()> {
        let mut client = self.client.clone();
        let mut url = self.url.clone();
        let qs = serde_qs::to_string(&params)?;
        url.set_query(Some(&qs));
        tracing::debug!(%url, "creating database");
        json_request::<(), ()>(
            &mut client,
            Method::PUT,
            self.url.clone(),
            &self.credentials,
            None,
        )
        .await
    }

    #[tracing::instrument(skip(self))]
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

    pub fn documents(&self) -> Documents<'_, C> {
        Documents::new(&self.client, &self.url, &self.credentials)
    }
}
