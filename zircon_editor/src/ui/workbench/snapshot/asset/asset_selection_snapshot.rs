use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::{
    AssetOperationProjectionSnapshot, AssetReferenceSnapshot, AssetSubassetSnapshot,
    AssetTypeProjectionSnapshot,
};
use crate::core::asset::AssetSourceAuthority;

#[derive(Clone, Debug, Default)]
pub struct AssetSelectionSnapshot {
    pub uuid: Option<String>,
    pub display_name: String,
    pub locator: String,
    pub kind: Option<ResourceKind>,
    pub asset_type: AssetTypeProjectionSnapshot,
    pub preview_artifact_path: String,
    pub meta_path: String,
    pub toolkit_view_id: String,
    pub toolkit_open_operation: String,
    pub context_commands: Vec<AssetOperationProjectionSnapshot>,
    pub package_id: Option<String>,
    pub asset_unit: String,
    pub included_files: Vec<String>,
    pub subassets: Vec<AssetSubassetSnapshot>,
    pub diagnostics: Vec<String>,
    pub resource_state: Option<ResourceState>,
    pub resource_revision: Option<u64>,
    pub references: Vec<AssetReferenceSnapshot>,
    pub used_by: Vec<AssetReferenceSnapshot>,
}

impl AssetSelectionSnapshot {
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
