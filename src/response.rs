use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::document::Document;

#[derive(Debug, Deserialize)]
pub struct ServerInstanceInfo {
    pub couchdb: String,
    pub uuid: String,
    pub git_sha: String,
    pub version: String,
    pub vendor: ServerInstanceInfoVendor,
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerInstanceInfoVendor {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseInfo {
    pub cluster: ClusterReplicationParams,
    pub db_name: String,
    pub compact_running: bool,
    pub disk_format_version: usize,
    pub doc_count: usize,
    pub doc_del_count: usize,
    pub instance_start_time: String,
    pub props: HashMap<String, String>,
    pub purge_seq: String,
    pub sizes: Sizes,
    pub update_seq: String,
}

#[derive(Debug, Deserialize)]
pub struct ClusterReplicationParams {
    pub n: usize,
    pub q: usize,
    pub r: usize,
    pub w: usize,
}

#[derive(Debug, Deserialize)]
pub struct Sizes {
    pub active: usize,
    pub external: usize,
    pub file: usize,
}

#[derive(Debug, Deserialize)]
pub struct Ok {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct Rev {
    pub rev: String,
}

#[derive(Debug, Deserialize)]
pub struct DocumentOperation {
    pub id: String,
    pub ok: bool,
    pub rev: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tombstone {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev")]
    pub rev: String,
    #[serde(rename = "_deleted")]
    pub deleted: bool,
}

impl Document for Tombstone {
    fn id(&self) -> &str {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        Some(&self.rev)
    }

    fn set_id(&mut self, id: impl ToString) -> &mut Self {
        self.id = id.to_string();
        self
    }

    fn set_rev(&mut self, rev: impl ToString) -> &mut Self {
        self.rev = rev.to_string();
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct ViewResults<V, T> {
    pub offset: usize,
    pub total_rows: usize,
    pub update_seq: Option<String>,
    pub rows: Vec<ViewRow<V, T>>,
}

impl<V, T> ViewResults<V, T> {
    pub fn iter(&self) -> std::slice::Iter<ViewRow<V, T>> {
        self.rows.iter()
    }
}

impl<V, T> IntoIterator for ViewResults<V, T> {
    type Item = ViewRow<V, T>;

    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct ViewRow<V, T> {
    pub id: String,
    pub key: String,
    pub value: V,
    pub doc: Option<T>,
}
