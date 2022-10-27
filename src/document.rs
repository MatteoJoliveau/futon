use futon_core::{Credentials, FutonClient, FutonRequest, Service};
use http::{Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use url::Url;

use crate::{request::CopyDestination, response::DocumentOperation, FutonResult};

pub trait Document: Serialize + DeserializeOwned {
    fn id(&self) -> &str;
    fn rev(&self) -> Option<&str>;
    fn set_id(&mut self, id: impl ToString) -> &mut Self;
    fn set_rev(&mut self, rev: impl ToString) -> &mut Self;
}

pub struct Documents<'db> {
    client: &'db FutonClient,
    url: &'db Url,
    db_name: &'db str,
    credentials: &'db Credentials,
}

impl<'db> Documents<'db> {
    pub fn new(
        client: &'db FutonClient,
        url: &'db Url,
        db_name: &'db str,
        credentials: &'db Credentials,
    ) -> Self {
        Self {
            client,
            url,
            db_name,
            credentials,
        }
    }
}

impl<'db> Documents<'db> {
    #[tracing::instrument(skip(self))]
    pub async fn create<D: Document + Debug>(&self, mut doc: D) -> FutonResult<D> {
        debug_assert!(doc.rev().is_none(), "doc should not have a rev set when creating. Use Documents::create_or_update() instead");
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::POST)?
            .credentials(self.credentials.clone())
            .database(self.db_name)
            .json(&doc)?;

        let res = client.call(req).await?;

        let DocumentOperation { rev, .. } = res.error_for_status()?.into_body().json()?;

        doc.set_rev(rev);
        Ok(doc)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_or_update<D: Document + Debug>(&self, mut doc: D) -> FutonResult<D> {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::PUT)?
            .credentials(self.credentials.clone())
            .database(self.db_name)
            .document(doc.id(), doc.rev())
            .json(&doc)?;

        let res = client.call(req).await?;

        let DocumentOperation { rev, .. } = res.error_for_status()?.into_body().json()?;

        doc.set_rev(rev);
        Ok(doc)
    }

    #[tracing::instrument(skip(self))]
    pub async fn exists(&self, id: &str) -> FutonResult<bool> {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::HEAD)?
            .credentials(self.credentials.clone())
            .database(self.db_name)
            .document(id, None);

        let res = client.call(req).await?;

        Ok(res.status() != StatusCode::NOT_FOUND)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get<D: Document>(&self, id: &str) -> FutonResult<Option<D>> {
        self.fetch(id, None).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_rev<D: Document>(&self, id: &str, rev: &str) -> FutonResult<Option<D>> {
        self.fetch(id, Some(rev)).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find<D: Document>(&self, id: &str, rev: Option<&str>) -> FutonResult<Option<D>> {
        self.fetch(id, rev).await
    }

    #[inline]
    async fn fetch<D: Document>(&self, id: &str, rev: Option<&str>) -> FutonResult<Option<D>> {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::GET)?
            .credentials(self.credentials.clone())
            .database(self.db_name)
            .document(id, rev);

        let res = client.call(req).await?;

        if res.is_not_found() {
            return Ok(None);
        }

        let doc = res.error_for_status()?.into_body().json()?;
        Ok(Some(doc))
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete<D: Document + Debug>(&self, doc: D) -> FutonResult<D> {
        let mut doc = match doc.rev() {
            Some(_rev) => doc,
            None => match self.get(doc.id()).await? {
                Some(doc) => doc,
                None => return Ok(doc),
            },
        };

        let _rev = doc.rev().unwrap();
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .method(Method::DELETE)?
            .credentials(self.credentials.clone())
            .database(self.db_name)
            .document(doc.id(), doc.rev());

        let res = client.call(req).await?;

        let DocumentOperation { rev, .. } = res.error_for_status()?.into_body().json()?;

        doc.set_rev(rev);
        Ok(doc)
    }

    #[tracing::instrument(skip(self))]
    pub async fn copy<D: Document + Debug>(
        &self,
        mut doc: D,
        destination: CopyDestination,
    ) -> FutonResult<D> {
        let mut client = self.client.clone();

        let rev = destination
            .rev
            .map(|rev| format!("?rev={rev}"))
            .unwrap_or_default();
        let destination = format!("{}{}", destination.id, rev);

        let req = FutonRequest::new(self.url.clone())?
            .method("COPY")?
            .header("destination", destination)?
            .credentials(self.credentials.clone())
            .database(self.db_name)
            .document(doc.id(), doc.rev());

        let res = client.call(req).await?;

        let DocumentOperation { id, rev, .. } = res.error_for_status()?.into_body().json()?;

        doc.set_id(id).set_rev(rev);
        Ok(doc)
    }
}
