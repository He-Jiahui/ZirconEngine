use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportGeneratedFile {
    pub path: String,
    pub purpose: String,
    pub contents: String,
}
