use futon_core::{Credentials, FutonClient, FutonRequest, Service};
use http::{Method, StatusCode};
use url::Url;

use crate::{
    ddoc::DesignDocuments,
    document::{Document, Documents},
    error::FutonError,
    request::{DatabaseCreationParams, ViewParams},
    response::{DatabaseInfo, Rev, ViewResults},
    FutonResult,
};
use std::fmt::Debug;

const NAME_REGEX: &str = r#"^[a-z][a-z0-9_$()+/-]*$"#;

pub struct Database {
    client: FutonClient,
    url: Url,
    name: String,
    credentials: Credentials,
}

impl Database {
    pub(crate) fn new(
        client: FutonClient,
        url: Url,
        name: impl ToString,
        credentials: Credentials,
    ) -> FutonResult<Self> {
        let re = regex::Regex::new(NAME_REGEX).unwrap();
        let name = name.to_string();
        if !re.is_match(&name) {
            return Err(FutonError::InvalidDatabaseName(name));
        }

        Ok(Self {
            client,
            url,
            name,
            credentials,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn exists(&self) -> FutonResult<bool> {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::HEAD)?
            .credentials(self.credentials.clone())
            .database(&self.name);

        let res = client.call(req).await?;

        Ok(res.status() != StatusCode::NOT_FOUND)
    }

    #[tracing::instrument(skip(self))]
    pub async fn info(&self) -> FutonResult<DatabaseInfo> {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .credentials(self.credentials.clone())
            .database(&self.name);

        let res = client.call(req).await?;

        let info = res.error_for_status()?.into_body().json()?;
        Ok(info)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, params: DatabaseCreationParams) -> FutonResult<()> {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::PUT)?
            .credentials(self.credentials.clone())
            .database(&self.name)
            .query_string(&params)?;

        tracing::debug!(%req, "creating database");

        client.call(req).await?.error_for_status()?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self) -> FutonResult<()> {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::DELETE)?
            .credentials(self.credentials.clone())
            .database(&self.name);

        client.call(req).await?.error_for_status()?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn all_docs<D: Document + Debug>(
        &self,
        params: ViewParams,
    ) -> FutonResult<ViewResults<Rev, D>> {
        self.design_docs(None)
            .execute_builtin_view("_all_docs", params)
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn all_docs_in_partition<D: Document + Debug>(
        &self,
        partition: String,
        params: ViewParams,
    ) -> FutonResult<ViewResults<Rev, D>> {
        self.design_docs(Some(partition))
            .execute_builtin_view("_all_docs", params)
            .await
    }

    #[inline]
    pub fn documents(&self) -> Documents<'_> {
        Documents::new(&self.client, &self.url, &self.name, &self.credentials)
    }

    #[inline]
    pub fn design_docs(&self, partition: Option<String>) -> DesignDocuments<'_> {
        DesignDocuments::new(
            &self.client,
            &self.url,
            partition,
            &self.name,
            &self.credentials,
        )
    }
}
