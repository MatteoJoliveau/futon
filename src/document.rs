use http::Method;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use url::Url;

use crate::{
    auth::Credentials,
    client::{json_request, maybe_json_request, Client},
    response::DocumentOperation,
    FutonResult,
};

pub trait Document: Serialize + DeserializeOwned {
    fn id(&self) -> &str;
    fn rev(&self) -> Option<&str>;
    fn set_rev(&mut self, rev: impl ToString) -> &mut Self;
}

pub struct Documents<'db, C> {
    client: &'db C,
    url: &'db Url,
    credentials: &'db Credentials,
}

impl<'db, C> Documents<'db, C> {
    pub fn new(client: &'db C, url: &'db Url, credentials: &'db Credentials) -> Self {
        Self {
            client,
            url,
            credentials,
        }
    }
}

impl<'db, C: Client> Documents<'db, C> {
    fn doc_url(&self, id: &str) -> Url {
        let mut url = self.url.clone();
        url.path_segments_mut().unwrap().push(id);
        url
    }

    #[tracing::instrument(skip(self))]
    pub async fn create<D: Document + Debug>(&self, mut doc: D) -> FutonResult<D> {
        debug_assert!(doc.rev().is_none(), "doc should not have a rev set when creating. Use Documents::create_or_update() instead");
        let mut client = self.client.clone();
        let DocumentOperation { rev, .. } = json_request::<&D, DocumentOperation>(
            &mut client,
            Method::POST,
            self.url.clone(),
            self.credentials,
            Some(&doc),
        )
        .await?;

        doc.set_rev(rev);
        Ok(doc)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_or_update<D: Document + Debug>(&self, mut doc: D) -> FutonResult<D> {
        let mut client = self.client.clone();
        let mut url = self.doc_url(doc.id());

        if let Some(rev) = doc.rev() {
            url.query_pairs_mut().append_pair("rev", rev);
        }
        let DocumentOperation { rev, .. } = json_request::<&D, DocumentOperation>(
            &mut client,
            Method::PUT,
            url,
            self.credentials,
            Some(&doc),
        )
        .await?;

        doc.set_rev(rev);
        Ok(doc)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get<D: Document>(&self, id: &str, rev: Option<&str>) -> FutonResult<Option<D>> {
        let mut client = self.client.clone();
        let mut url = self.doc_url(id);

        if let Some(rev) = rev {
            url.query_pairs_mut().append_pair("rev", rev);
        }

        maybe_json_request::<(), D>(&mut client, Method::GET, url, self.credentials, None).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete<D: Document + Debug>(&self, mut doc: D) -> FutonResult<D> {
        let rev = doc.rev().unwrap();
        let mut client = self.client.clone();
        let mut url = self.doc_url(doc.id());
        url.query_pairs_mut().append_pair("rev", rev);

        let DocumentOperation { rev, .. } = json_request::<(), DocumentOperation>(
            &mut client,
            Method::DELETE,
            url,
            self.credentials,
            None,
        )
        .await?;

        doc.set_rev(rev);
        Ok(doc)
    }
}
