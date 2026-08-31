use serde::{Deserialize, Serialize};

use crate::asset::assets::{
    MaterialAssetManagementRecordSet, MeshAssetManagementRecordSet, ModelAssetManagementRecordSet,
    SceneAssetManagementRecordSet, SceneEntityManagementRecordSet, ShaderAssetManagementRecordSet,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;

mod family;
mod record_sets;

/// Header totals for the combined asset management read model.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementRecordSetSummary {
    pub managed_record_count: usize,
    pub degraded_record_count: usize,
    pub model_count: usize,
    pub model_mesh_referenced_model_count: usize,
    pub model_mesh_reference_count: usize,
    pub mesh_count: usize,
    pub valid_mesh_count: usize,
    pub invalid_mesh_count: usize,
    pub mesh_morph_target_count: usize,
    pub mesh_morph_target_attribute_count: usize,
    pub scene_count: usize,
    pub scene_entity_count: usize,
    pub entity_count: usize,
    pub active_entity_count: usize,
    pub root_entity_count: usize,
    pub entity_direct_reference_count: usize,
    pub entity_camera_count: usize,
    pub entity_mesh_instance_count: usize,
    pub entity_direct_mesh_reference_count: usize,
    pub entity_mesh_primitive_binding_count: usize,
    pub entity_morph_weight_count: usize,
    pub entity_mesh_material_binding_count: usize,
    pub entity_collider_material_binding_count: usize,
    pub entity_light_count: usize,
    pub entity_physics_component_count: usize,
    pub entity_animation_binding_count: usize,
    pub entity_terrain_count: usize,
    pub entity_tilemap_count: usize,
    pub entity_prefab_instance_count: usize,
    pub material_count: usize,
    pub material_ready_count: usize,
    pub material_degraded_count: usize,
    pub material_issue_row_count: usize,
    pub material_property_override_count: usize,
    pub material_texture_slot_count: usize,
    pub material_texture_reference_count: usize,
    pub material_fallback_texture_slot_count: usize,
    pub material_validation_error_count: usize,
    pub material_validation_diagnostic_count: usize,
    pub material_direct_reference_count: usize,
    pub prepared_material_count: usize,
    pub prepared_material_ready_count: usize,
    pub prepared_material_degraded_count: usize,
    pub prepared_material_issue_row_count: usize,
    pub shader_count: usize,
    pub shader_ready_count: usize,
    pub shader_not_ready_count: usize,
    pub shader_issue_row_count: usize,
    pub shader_validation_diagnostic_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssetManagementFamilyKind {
    Model,
    Mesh,
    Scene,
    Entity,
    Material,
    Shader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetManagementFamilyStatus {
    Empty,
    Ready,
    Degraded,
}

/// Compact row for the top-level asset-family management overview.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilySummary {
    pub kind: AssetManagementFamilyKind,
    pub status: AssetManagementFamilyStatus,
    pub total_record_count: usize,
    pub ready_record_count: usize,
    pub degraded_record_count: usize,
    pub issue_row_count: usize,
}

/// Status buckets for the fixed top-level asset families.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyStatusIndex {
    pub empty: Vec<AssetManagementFamilyKind>,
    pub ready: Vec<AssetManagementFamilyKind>,
    pub degraded: Vec<AssetManagementFamilyKind>,
}

/// Row-bearing companion for one top-level family status bucket.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyStatusView {
    pub status: AssetManagementFamilyStatus,
    pub families: Vec<AssetManagementFamilyKind>,
    pub rows: Vec<AssetManagementFamilySummary>,
    pub total_record_count: usize,
    pub ready_record_count: usize,
    pub degraded_record_count: usize,
    pub issue_row_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetManagementFamilyIssueBucket {
    Clean,
    WithIssues,
}

/// Issue-row buckets for top-level asset-family drilldown navigation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyIssueIndex {
    pub clean: Vec<AssetManagementFamilyKind>,
    pub with_issues: Vec<AssetManagementFamilyKind>,
}

/// Row-bearing companion for one top-level family issue bucket.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyIssueView {
    pub bucket: AssetManagementFamilyIssueBucket,
    pub families: Vec<AssetManagementFamilyKind>,
    pub rows: Vec<AssetManagementFamilySummary>,
    pub issue_row_count: usize,
}

/// Lightweight top-level state for management headers and navigation chrome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementOverview {
    pub summary: AssetManagementRecordSetSummary,
    pub families: Vec<AssetManagementFamilySummary>,
    #[serde(default)]
    pub family_status_index: AssetManagementFamilyStatusIndex,
    #[serde(default)]
    pub family_issue_index: AssetManagementFamilyIssueIndex,
}

/// One payload for renderer/editor panels that need all asset-management lists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetManagementRecordSets {
    pub summary: AssetManagementRecordSetSummary,
    pub families: Vec<AssetManagementFamilySummary>,
    #[serde(default)]
    pub family_status_index: AssetManagementFamilyStatusIndex,
    #[serde(default)]
    pub family_issue_index: AssetManagementFamilyIssueIndex,
    pub models: ModelAssetManagementRecordSet,
    pub meshes: MeshAssetManagementRecordSet,
    pub scenes: SceneAssetManagementRecordSet,
    pub scene_entities: SceneEntityManagementRecordSet,
    pub material_assets: MaterialAssetManagementRecordSet,
    /// Renderer-prepared material detail records kept beside asset-level material rows.
    pub materials: RenderMaterialManagementRecordSet,
    pub shaders: ShaderAssetManagementRecordSet,
}
