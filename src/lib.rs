use auth::Credentials;

use client::Client;
use db::Database;
use error::FutonError;

use meta::Meta;

use url::Url;

pub mod auth;
pub mod client;
pub mod db;
pub mod document;
pub mod error;
pub mod meta;
pub mod request;
pub mod response;

pub type FutonResult<T> = std::result::Result<T, FutonError>;

#[cfg(feature = "hyper")]
pub type DefaultClient = client::hyper::HyperClient;

pub struct Futon<C = DefaultClient> {
    client: C,
    url: Url,
    credentials: Credentials,
}

#[cfg(feature = "hyper")]
impl Futon<DefaultClient> {
    pub fn new<U: Into<Url>>(url: U) -> Self {
        let url = url.into();
        let username = url.username();
        let credentials = if username.is_empty() {
            None
        } else {
            Some(username).zip(url.password())
        }
        .map(|(u, p)| Credentials::basic(u, p))
        .unwrap_or_default();
        Self::new_with_credentials(url, credentials)
    }

    pub fn new_with_credentials<U: Into<Url>>(url: U, credentials: Credentials) -> Self {
        let mut url = url.into();
        url.set_username("").unwrap();
        url.set_password(None).unwrap();
        Self {
            client: DefaultClient::new(),
            url,
            credentials,
        }
    }
}

impl<C: Client> Futon<C> {
    pub fn meta(&self) -> Meta<C> {
        Meta::new(
            self.client.clone(),
            self.url.clone(),
            self.credentials.clone(),
        )
    }

    pub fn db(&self, name: impl AsRef<str>) -> FutonResult<Database<C>> {
        Database::new(
            self.client.clone(),
            self.url.clone(),
            name.as_ref(),
            self.credentials.clone(),
        )
    }
}
