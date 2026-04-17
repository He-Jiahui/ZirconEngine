use serde::{Deserialize, Serialize};
use zircon_resource::ResourceKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetPipelineInfo {
    pub default_worker_count: usize,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetChangeKind {
    Added,
    Modified,
    Removed,
    Renamed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetChangeRecord {
    pub kind: AssetChangeKind,
    pub uri: String,
    pub previous_uri: Option<String>,
}
