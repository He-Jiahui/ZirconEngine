use crate::core::resource::ResourceKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetStatusRecord {
    pub id: String,
    pub uri: String,
    pub kind: ResourceKind,
    pub artifact_uri: Option<String>,
    /// 仅当注册表状态为 Ready 时为真；Pending、Reloading 和 Error 都返回 false。
    pub imported: bool,
    pub source_hash: String,
    pub importer_id: String,
    pub importer_version: u32,
    pub config_hash: String,
}
