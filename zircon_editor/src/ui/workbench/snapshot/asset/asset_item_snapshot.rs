use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::AssetTypeProjectionSnapshot;
use crate::core::asset::AssetSourceAuthority;

#[derive(Clone, Debug)]
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
    pub fn source_authority(&self) -> AssetSourceAuthority {
        AssetSourceAuthority::from_locator_str(self.asset_type.source_write_policy, &self.locator)
            .unwrap_or_default()
    }
}
