use auth::Credentials;
use client::{hyper::HyperClient, Client};
use db::Database;
use error::FutonError;
use meta::Meta;
use url::Url;

pub mod auth;
pub mod client;
pub mod db;
pub mod error;
pub mod meta;
pub mod response;

pub type FutonResult<T> = std::result::Result<T, FutonError>;

pub type DefaultClient = HyperClient;

pub struct Futon<C = DefaultClient> {
    client: C,
    url: Url,
    credentials: Credentials,
}

#[cfg(feature = "hyper")]
impl Futon<DefaultClient> {
    pub fn new<U: Into<Url>>(url: U) -> Self {
        let mut url: Url = url.into();
        let username = url.username();
        let credentials = if username.is_empty() {
            None
        } else {
            Some(username).zip(url.password())
        }
        .map(|(u, p)| Credentials::basic(u, p))
        .unwrap_or_default();
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
        Ok(Database::new(
            self.client.clone(),
            self.url.join(name.as_ref())?,
            self.credentials.clone(),
        ))
    }
}
