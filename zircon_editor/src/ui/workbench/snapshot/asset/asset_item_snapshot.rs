use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::AssetTypeProjectionSnapshot;
use crate::core::asset::AssetSourceAuthority;

#[derive(Clone, Debug)]
/// 可见资产代次中的一行；uuid/locator用于查找与拖动，运行时状态可在资源尚未加载时缺席。
pub struct AssetItemSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub file_name: String,
    pub extension: String,
    pub kind: ResourceKind,
    pub asset_type: AssetTypeProjectionSnapshot,
    pub preview_artifact_path: String,
    pub dirty: bool,
    pub diagnostics: Vec<String>,
    pub selected: bool,
    pub resource_state: Option<ResourceState>,
    pub resource_revision: Option<u64>,
}

impl AssetItemSnapshot {
    /// 结合类型写入策略与合法locator解释来源；解析失败保守返回只读临时来源。
    pub fn source_authority(&self) -> AssetSourceAuthority {
        AssetSourceAuthority::from_locator_str(self.asset_type.source_write_policy, &self.locator)
            .unwrap_or_default()
    }
}
