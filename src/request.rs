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

#[derive(Debug, Serialize)]
pub struct ViewParams {
    pub conflicts: bool,
    pub descending: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_key: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_key_doc_id: Option<String>,
    pub group: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_level: Option<usize>,
    pub include_docs: bool,
    pub attachments: bool,
    #[serde(rename = "att_encoding_info")]
    pub attachments_encoding_info: bool,
    pub inclusive_end: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    pub reduce: bool,
    pub skip: usize,
    pub sorted: bool,
    pub stable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_key: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_key_doc_id: Option<String>,
    pub update: Update,
    pub update_seq: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    True,
    False,
    Lazy,
}

impl Default for Update {
    fn default() -> Self {
        Self::True
    }
}

impl Default for ViewParams {
    fn default() -> Self {
        Self {
            conflicts: false,
            descending: false,
            end_key: None,
            end_key_doc_id: None,
            group: false,
            group_level: None,
            include_docs: false,
            attachments: false,
            attachments_encoding_info: false,
            inclusive_end: false,
            key: None,
            keys: None,
            limit: None,
            reduce: false,
            skip: 0,
            sorted: true,
            stable: false,
            start_key: None,
            start_key_doc_id: None,
            update: Update::default(),
            update_seq: false,
        }
    }
}
