use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::{
    AssetOperationProjectionSnapshot, AssetReferenceSnapshot, AssetSubassetSnapshot,
    AssetTypeProjectionSnapshot,
};
use crate::core::asset::AssetSourceAuthority;

#[derive(Clone, Debug, Default)]
/// 当前资产的跨面详情快照；基础目录信息、运行时状态和注册类型工具信息在发布前汇合。
pub struct AssetSelectionSnapshot {
    pub uuid: Option<String>,
    pub display_name: String,
    pub locator: String,
    pub kind: Option<ResourceKind>,
    pub asset_type: AssetTypeProjectionSnapshot,
    pub preview_artifact_path: String,
    pub meta_path: String,
    pub toolkit_view_id: String,
    /// 注册类型声明的打开路径；须回操作中心执行，不能把此快照当可执行授权。
    pub toolkit_open_operation: String,
    pub context_commands: Vec<AssetOperationProjectionSnapshot>,
    pub package_id: Option<String>,
    pub asset_unit: String,
    pub included_files: Vec<String>,
    pub subassets: Vec<AssetSubassetSnapshot>,
    pub diagnostics: Vec<String>,
    pub resource_state: Option<ResourceState>,
    pub resource_revision: Option<u64>,
    /// 当前选择的直接引用，与used_by方向相反；目标可能尚未在项目catalog中解析。
    pub references: Vec<AssetReferenceSnapshot>,
    pub used_by: Vec<AssetReferenceSnapshot>,
}

impl AssetSelectionSnapshot {
    /// 命令评估的来源/写入策略输入；无合法locator时返回空，由消费方采用只读回退。
    pub fn source_authority(&self) -> Option<AssetSourceAuthority> {
        (!self.locator.is_empty())
            .then(|| {
                AssetSourceAuthority::from_locator_str(
                    self.asset_type.source_write_policy,
                    &self.locator,
                )
            })
            .transpose()
            .ok()
            .flatten()
    }
}
