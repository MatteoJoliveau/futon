mod common;

use futon::{error::FutonError, request::DatabaseCreationParams};

#[tokio::test]
async fn it_creates_and_deletes_a_db() {
    tracing_subscriber::fmt::init();

    common::with_couchdb(|name, futon| async move {
        let db = futon.db(name)?;

        let err = db.info().await.unwrap_err();
        assert!(err.is_not_found());

        let exists = db.exists().await?;
        assert!(!exists);

        db.create(Default::default()).await?;

        let info = db.info().await?;

        assert_eq!(&info.db_name, &name);
        assert!(info.props.is_empty());

        let exists = db.exists().await?;
        assert!(exists);

        db.delete().await?;

        let err = db.info().await.unwrap_err();
        assert!(err.is_not_found());

        let exists = db.exists().await?;
        assert!(!exists);

        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn it_creates_a_partitioned_db() {
    tracing_subscriber::fmt::init();

    common::with_couchdb(|name, futon| async move {
        let db = futon.db(name)?;
        db.create(DatabaseCreationParams::partitioned()).await?;

        let info = db.info().await?;
        tracing::warn!("{info:?}");
        Ok(())
    })
    .await
    .unwrap();
}

#[test]
fn it_validates_db_names() {
    let futon = futon::Futon::new("http://example.com".parse::<url::Url>().unwrap());

    for name in [
        "",
        "1-test",
        "$test",
        "invalid%name",
        "invalid?name",
        "INVALID_NAME",
        "invalid.name",
        "invalid,name",
        "invalidname!",
        "invalid[name]",
        "invalid{name}",
    ] {
        let err = futon.db(name).err().expect(name);
        assert!(matches!(err, FutonError::InvalidDatabaseName(_)));
        assert!(err
            .to_string()
            .contains("https://docs.couchdb.org/en/stable/api/database/common.html#put--db"));
    }
}
