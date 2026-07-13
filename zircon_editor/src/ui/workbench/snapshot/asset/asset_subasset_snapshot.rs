use zircon_runtime_interface::resource::ResourceKind;

use super::AssetTypeProjectionSnapshot;

#[derive(Clone, Debug)]
pub struct AssetSubassetSnapshot {
    pub uuid: String,
    pub locator: String,
    pub kind: ResourceKind,
    pub asset_type: AssetTypeProjectionSnapshot,
    pub artifact_locator: Option<String>,
    pub dependency_locators: Vec<String>,
}
