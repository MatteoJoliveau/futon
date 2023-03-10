use std::{convert::Infallible, fmt::Display, str::FromStr};

use futon_core::{Credentials, FutonClient, FutonRequest, Service};
use http::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

use crate::{
    document::{Document, Documents},
    request::ViewParams,
    response::ViewResults,
    FutonResult,
};

#[derive(Debug)]
pub enum QueryServer {
    JavaScript,
    Erlang,
    Custom(String),
}

impl Default for QueryServer {
    fn default() -> Self {
        Self::JavaScript
    }
}

impl FromStr for QueryServer {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let qs = match s.to_lowercase().as_str() {
            "javascript" => Self::JavaScript,
            "erlang" => Self::Erlang,
            lang => Self::Custom(lang.to_string()),
        };

        Ok(qs)
    }
}

impl Display for QueryServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            QueryServer::JavaScript => "javascript",
            QueryServer::Erlang => "erlang",
            QueryServer::Custom(s) => s,
        };

        Display::fmt(s, f)
    }
}

impl Serialize for QueryServer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for QueryServer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let qs = Self::from_str(&s).unwrap();
        Ok(qs)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DesignDocument {
    id: String,
    rev: Option<String>,
    language: QueryServer,
}

impl DesignDocument {
    pub fn new(name: &str, rev: Option<impl Into<String>>, language: QueryServer) -> Self {
        let name = name.strip_prefix("_design").unwrap_or(name);
        Self {
            id: format!("_design/{name}"),
            rev: rev.map(Into::into),
            language,
        }
    }
}

impl Document for DesignDocument {
    fn id(&self) -> &str {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_id(&mut self, id: impl ToString) -> &mut Self {
        self.id = id.to_string();
        self
    }

    fn set_rev(&mut self, rev: impl ToString) -> &mut Self {
        self.rev = Some(rev.to_string());
        self
    }
}

pub struct DesignDocuments<'db> {
    client: &'db FutonClient,
    url: &'db Url,
    partition: Option<String>,
    db_name: &'db str,
    credentials: &'db Credentials,
}

impl<'db> DesignDocuments<'db> {
    pub fn new(
        client: &'db FutonClient,
        url: &'db Url,
        partition: Option<String>,
        db_name: &'db str,
        credentials: &'db Credentials,
    ) -> Self {
        Self {
            client,
            url,
            partition,
            db_name,
            credentials,
        }
    }
}

impl<'db> DesignDocuments<'db> {
    #[inline]
    fn docs(&self) -> Documents<'_> {
        Documents::new(self.client, self.url, self.db_name, self.credentials)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, doc: DesignDocument) -> FutonResult<DesignDocument> {
        self.docs().create(doc).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_or_update(&self, doc: DesignDocument) -> FutonResult<DesignDocument> {
        self.docs().create_or_update(doc).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute_view<V, T>(
        &self,
        ddoc: &str,
        view: &str,
    ) -> FutonResult<ViewResults<V, T>> {
        todo!()
    }

    pub async fn execute_builtin_view<V, T>(
        &self,
        view: &str,
        params: ViewParams,
    ) -> FutonResult<ViewResults<V, T>>
    where
        V: DeserializeOwned,
        T: DeserializeOwned,
    {
        let mut client = self.client.clone();

        let req = FutonRequest::new(self.url.clone())?
            .credentials(self.credentials.clone())
            .method(Method::POST)?
            .database(self.db_name)
            .document(view, None)
            .json(params)?;

        let req = match self.partition {
            Some(ref partition) => req.partition(partition),
            None => req,
        };

        let res = client.call(req).await?;

        let results = res.error_for_status()?.into_body().json()?;
        Ok(results)
    }
}
