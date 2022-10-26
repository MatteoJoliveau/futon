mod common;

use common::TestDocument;

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

#[tokio::test]
async fn it_lists_all_documents() {
    tracing_subscriber::fmt::init();

    common::with_db(|db| async move {
        let doc = TestDocument {
            id: "test".to_string(),
            rev: None,
            message: "Hello Futon!".to_string(),
        };

        let docs = db.documents();
        let doc = docs.create(doc).await?;

        let all_docs = db.all_docs::<TestDocument>(Default::default()).await?;
        assert_eq!(all_docs.offset, 0);
        assert_eq!(all_docs.total_rows, 1);
        assert!(all_docs.update_seq.is_none());
        let mut iter = all_docs.into_iter();
        assert_eq!(iter.next().and_then(|row| row.doc), Some(doc));

        Ok(())
    })
    .await
    .unwrap();
}
