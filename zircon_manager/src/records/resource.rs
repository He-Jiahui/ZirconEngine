use serde::{Deserialize, Serialize};

use super::AssetRecordKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStateRecord {
    Pending,
    Ready,
    Error,
    Reloading,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceStatusRecord {
    pub id: String,
    pub locator: String,
    pub kind: AssetRecordKind,
    pub artifact_locator: Option<String>,
    pub revision: u64,
    pub state: ResourceStateRecord,
    pub dependency_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceChangeKind {
    Added,
    Updated,
    Removed,
    Renamed,
    ReloadFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceChangeRecord {
    pub kind: ResourceChangeKind,
    pub id: String,
    pub locator: Option<String>,
    pub previous_locator: Option<String>,
    pub revision: u64,
}
