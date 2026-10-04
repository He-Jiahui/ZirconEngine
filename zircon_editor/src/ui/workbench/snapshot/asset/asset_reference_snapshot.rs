use zircon_runtime_interface::resource::ResourceKind;

use super::AssetTypeProjectionSnapshot;

#[derive(Clone, Debug, Default)]
/// 引用/被引用目标行；未知目标仍可展示，但只有已知项目资产允许走资产拖动链。
pub struct AssetReferenceSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub kind: Option<ResourceKind>,
    pub asset_type: Option<AssetTypeProjectionSnapshot>,
    pub known_project_asset: bool,
}
