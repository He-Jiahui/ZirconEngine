use zircon_resource::{ResourceKind, ResourceState};

use super::AssetReferenceSnapshot;

#[derive(Clone, Debug, Default)]
pub struct AssetSelectionSnapshot {
    pub uuid: Option<String>,
    pub display_name: String,
    pub locator: String,
    pub kind: Option<ResourceKind>,
    pub preview_artifact_path: String,
    pub meta_path: String,
    pub adapter_key: String,
    pub diagnostics: Vec<String>,
    pub resource_state: Option<ResourceState>,
    pub resource_revision: Option<u64>,
    pub references: Vec<AssetReferenceSnapshot>,
    pub used_by: Vec<AssetReferenceSnapshot>,
}
