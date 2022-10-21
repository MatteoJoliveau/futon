use serde::Serialize;

use crate::document::Document;

#[derive(Debug, Default, Serialize)]
pub struct DatabaseCreationParams {
    pub q: Option<usize>,
    pub n: Option<usize>,
    pub partitioned: bool,
}

impl DatabaseCreationParams {
    pub fn partitioned() -> Self {
        Self {
            partitioned: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CopyDestination {
    pub id: String,
    pub rev: Option<String>,
}

impl CopyDestination {
    pub fn new(id: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            rev: None,
        }
    }

    pub fn existing(id: impl ToString, rev: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            rev: Some(rev.to_string()),
        }
    }

    pub fn from_doc<D: Document>(doc: &D) -> Self {
        Self {
            id: doc.id().into(),
            rev: doc.rev().map(Into::into),
        }
    }
}

impl<D: Document> From<D> for CopyDestination {
    fn from(doc: D) -> Self {
        Self::from_doc(&doc)
    }
}
