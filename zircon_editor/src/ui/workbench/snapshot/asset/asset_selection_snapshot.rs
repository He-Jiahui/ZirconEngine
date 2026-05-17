use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::{AssetReferenceSnapshot, AssetSubassetSnapshot};

#[derive(Clone, Debug, Default)]
pub struct AssetSelectionSnapshot {
    pub uuid: Option<String>,
    pub display_name: String,
    pub locator: String,
    pub kind: Option<ResourceKind>,
    pub preview_artifact_path: String,
    pub meta_path: String,
    pub adapter_key: String,
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
