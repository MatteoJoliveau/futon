use std::future::Future;

use futon::{db::Database, Credentials, Futon};

mod document;
mod image;

pub use document::TestDocument;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub async fn with_couchdb<Test, Fut>(test: Test) -> TestResult
where
    Fut: Future<Output = TestResult>,
    Test: FnOnce(&'static str, Futon) -> Fut,
{
    let name = concat!(env!("CARGO_PKG_NAME"), "-test");
    #[cfg(not(any(feature = "test-docker", feature = "test-podman")))]
    let client = testcontainers::clients::Cli::default();
    #[cfg(feature = "test-docker")]
    let client = testcontainers::clients::Cli::docker();
    #[cfg(feature = "test-podman")]
    let client = testcontainers::clients::Cli::podman();

    let couchdb = client.run(image::CouchDb::default());
    let url = format!("http://127.0.0.1:{}", couchdb.get_host_port_ipv4(5984))
        .parse::<url::Url>()
        .unwrap();

    let futon =
        Futon::new_with_credentials(url, Credentials::basic(image::USERNAME, image::PASSWORD));

    test(name, futon).await?;

    Ok(())
}

pub async fn with_db<Test, Fut>(test: Test) -> TestResult
where
    Fut: Future<Output = TestResult>,
    Test: FnOnce(Database) -> Fut,
{
    with_couchdb(|name, futon| async move {
        let db = futon.db(name)?;
        db.create(Default::default()).await?;
        test(db).await
    })
    .await
}
