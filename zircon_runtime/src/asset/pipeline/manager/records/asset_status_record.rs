use crate::core::resource::ResourceKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetStatusRecord {
    pub id: String,
    pub uri: String,
    pub kind: ResourceKind,
    pub artifact_uri: Option<String>,
    pub imported: bool,
    pub source_hash: String,
    pub importer_version: u32,
    pub config_hash: String,
}
