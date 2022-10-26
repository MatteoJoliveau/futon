use futon::document::Document;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestDocument {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    pub message: String,
}

impl Document for TestDocument {
    fn id(&self) -> &str {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_id(&mut self, id: impl ToString) -> &mut Self {
        self.id = id.to_string();
        self
    }

    fn set_rev(&mut self, rev: impl ToString) -> &mut Self {
        self.rev = Some(rev.to_string());
        self
    }
}
