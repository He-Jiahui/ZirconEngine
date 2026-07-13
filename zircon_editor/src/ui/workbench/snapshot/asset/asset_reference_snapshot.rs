use zircon_runtime_interface::resource::ResourceKind;

use super::AssetTypeProjectionSnapshot;

#[derive(Clone, Debug, Default)]
pub struct AssetReferenceSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub kind: Option<ResourceKind>,
    pub asset_type: Option<AssetTypeProjectionSnapshot>,
    pub known_project_asset: bool,
}
