pub use futon_core::Credentials;

use db::Database;
use error::FutonError;

use futon_core::FutonClient;
use meta::Meta;

use url::Url;

pub mod db;
pub mod ddoc;
pub mod document;
pub mod error;
pub mod meta;
pub mod request;
pub mod response;

pub type FutonResult<T> = std::result::Result<T, FutonError>;

pub struct Futon {
    client: FutonClient,
    url: Url,
    credentials: Credentials,
}

impl Futon {
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
            client: FutonClient::default(),
            url,
            credentials,
        }
    }
}

impl Futon {
    pub fn meta(&self) -> Meta {
        Meta::new(
            self.client.clone(),
            self.url.clone(),
            self.credentials.clone(),
        )
    }

    pub fn db(&self, name: impl AsRef<str>) -> FutonResult<Database> {
        Database::new(
            self.client.clone(),
            self.url.clone(),
            name.as_ref(),
            self.credentials.clone(),
        )
    }
}
