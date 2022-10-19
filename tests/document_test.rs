use futon::{document::Document, error::FutonError, response::Tombstone};
use serde::{Deserialize, Serialize};

mod common;
use common::with_db;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestDocument {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    message: String,
}

impl Document for TestDocument {
    fn id(&self) -> &str {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_rev(&mut self, rev: impl ToString) -> &mut Self {
        self.rev = Some(rev.to_string());
        self
    }
}

#[tokio::test]
async fn it_creates_a_new_document() {
    tracing_subscriber::fmt::init();

    with_db(|db| async move {
        let doc = TestDocument {
            id: "test".to_string(),
            rev: None,
            message: "Hello Futon!".to_string(),
        };

        let docs = db.documents();
        let doc = docs.create(doc).await?;

        assert_eq!(&doc.id, "test");
        assert!(doc.rev.is_some());
        assert_eq!(&doc.message, "Hello Futon!");
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(
    expected = "doc should not have a rev set when creating. Use Documents::create_or_update() instead"
)]
async fn it_panics_when_creating_an_existing_document_on_debug() {
    tracing_subscriber::fmt::init();

    with_db(|db| async move {
        let doc = TestDocument {
            id: "test".to_string(),
            rev: None,
            message: "Hello Futon!".to_string(),
        };

        let docs = db.documents();
        let doc = docs.create(doc).await?;

        assert_eq!(&doc.id, "test");
        assert!(doc.rev.is_some());
        assert_eq!(&doc.message, "Hello Futon!");

        docs.create(doc).await?; // should panic because of the debug_assert!(doc.rev.is_none())
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn it_errors_when_creating_an_existing_document() {
    tracing_subscriber::fmt::init();

    with_db(|db| async move {
        let doc = TestDocument {
            id: "test".to_string(),
            rev: None,
            message: "Hello Futon!".to_string(),
        };

        let docs = db.documents();
        let mut doc = docs.create(doc).await?;

        assert_eq!(&doc.id, "test");
        assert!(doc.rev.is_some());
        assert_eq!(&doc.message, "Hello Futon!");

        doc.rev = None;

        let err = docs.create(doc).await.unwrap_err();
        assert!(matches!(err, FutonError::Conflict(_)));
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn it_creates_and_updates_a_document() {
    tracing_subscriber::fmt::init();

    with_db(|db| async move {
        let doc = TestDocument {
            id: "test".to_string(),
            rev: None,
            message: "Hello Futon!".to_string(),
        };

        let docs = db.documents();
        let mut doc = docs.create_or_update(doc).await?;

        assert_eq!(&doc.id, "test");
        assert!(doc.rev.is_some());
        assert_eq!(&doc.message, "Hello Futon!");

        let old_rev = doc.rev.clone();

        doc.message = "Updated message".to_string();

        let doc = docs.create_or_update(doc).await?;
        assert_eq!(&doc.id, "test");
        assert_ne!(doc.rev, old_rev);
        assert_eq!(&doc.message, "Updated message");
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn it_fetches_a_document() {
    tracing_subscriber::fmt::init();

    with_db(|db| async move {
        let doc = TestDocument {
            id: "test".to_string(),
            rev: None,
            message: "Hello Futon!".to_string(),
        };

        let docs = db.documents();

        let nope = docs.get::<TestDocument>(doc.id(), doc.rev()).await?;
        assert!(nope.is_none());

        let doc = docs.create(doc).await?;

        let same_doc = docs.get::<TestDocument>(doc.id(), doc.rev()).await?;
        assert_eq!(Some(doc), same_doc);
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn it_deletes_a_document() {
    tracing_subscriber::fmt::init();

    with_db(|db| async move {
        let doc = TestDocument {
            id: "test".to_string(),
            rev: None,
            message: "Hello Futon!".to_string(),
        };

        let docs = db.documents();

        let doc = docs.create(doc).await?;

        let old_rev = doc.rev.clone();

        let deleted = docs.delete(doc).await?;
        assert_eq!(&deleted.id, "test");
        assert_ne!(deleted.rev, old_rev);
        assert_eq!(&deleted.message, "Hello Futon!");

        let tombstone = docs
            .get::<Tombstone>(deleted.id(), deleted.rev())
            .await?
            .unwrap();
        assert_eq!(tombstone.id(), deleted.id());
        assert_eq!(tombstone.rev(), deleted.rev());
        assert!(tombstone.deleted);

        let nope = docs.get::<TestDocument>(deleted.id(), None).await?;
        assert!(nope.is_none());
        Ok(())
    })
    .await
    .unwrap();
}
