use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityWindowTemplateSpec {
    pub document_id: String,
}

impl ActivityWindowTemplateSpec {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
        }
    }
}
