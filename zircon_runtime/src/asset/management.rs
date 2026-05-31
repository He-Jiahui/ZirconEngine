use serde::{Deserialize, Serialize};

use crate::asset::assets::{
    MaterialAssetManagementRecordSet, MeshAssetManagementRecordSet, ModelAssetManagementRecordSet,
    SceneAssetManagementRecordSet, SceneEntityManagementRecordSet, ShaderAssetManagementRecordSet,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;

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

/// Lightweight top-level state for management headers and navigation chrome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementOverview {
    pub summary: AssetManagementRecordSetSummary,
    pub families: Vec<AssetManagementFamilySummary>,
    pub family_status_index: AssetManagementFamilyStatusIndex,
}

/// One payload for renderer/editor panels that need all asset-management lists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetManagementRecordSets {
    pub summary: AssetManagementRecordSetSummary,
    pub families: Vec<AssetManagementFamilySummary>,
    pub family_status_index: AssetManagementFamilyStatusIndex,
    pub models: ModelAssetManagementRecordSet,
    pub meshes: MeshAssetManagementRecordSet,
    pub scenes: SceneAssetManagementRecordSet,
    pub scene_entities: SceneEntityManagementRecordSet,
    pub material_assets: MaterialAssetManagementRecordSet,
    /// Renderer-prepared material detail records kept beside asset-level material rows.
    pub materials: RenderMaterialManagementRecordSet,
    pub shaders: ShaderAssetManagementRecordSet,
}

impl AssetManagementFamilySummary {
    pub fn new(
        kind: AssetManagementFamilyKind,
        total_record_count: usize,
        ready_record_count: usize,
        degraded_record_count: usize,
        issue_row_count: usize,
    ) -> Self {
        let status = if total_record_count == 0 {
            AssetManagementFamilyStatus::Empty
        } else if degraded_record_count > 0 {
            AssetManagementFamilyStatus::Degraded
        } else {
            AssetManagementFamilyStatus::Ready
        };
        Self {
            kind,
            status,
            total_record_count,
            ready_record_count,
            degraded_record_count,
            issue_row_count,
        }
    }
}

impl AssetManagementOverview {
    pub fn from_summary(summary: AssetManagementRecordSetSummary) -> Self {
        let families = summary.family_summaries();
        let family_status_index = AssetManagementFamilyStatusIndex::from_families(&families);
        Self {
            summary,
            families,
            family_status_index,
        }
    }

    pub fn family_summaries(&self) -> &[AssetManagementFamilySummary] {
        &self.families
    }

    pub fn family_status_index(&self) -> &AssetManagementFamilyStatusIndex {
        &self.family_status_index
    }
}

impl AssetManagementFamilyStatusIndex {
    pub fn from_families(families: &[AssetManagementFamilySummary]) -> Self {
        let mut index = Self::default();
        for family in families {
            match family.status {
                AssetManagementFamilyStatus::Empty => index.empty.push(family.kind),
                AssetManagementFamilyStatus::Ready => index.ready.push(family.kind),
                AssetManagementFamilyStatus::Degraded => index.degraded.push(family.kind),
            }
        }
        index
    }

    pub fn total_family_count(&self) -> usize {
        self.empty.len() + self.ready.len() + self.degraded.len()
    }

    pub fn degraded_family_count(&self) -> usize {
        self.degraded.len()
    }

    pub fn has_degraded_families(&self) -> bool {
        !self.degraded.is_empty()
    }

    pub fn families_for_status(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> &[AssetManagementFamilyKind] {
        match status {
            AssetManagementFamilyStatus::Empty => &self.empty,
            AssetManagementFamilyStatus::Ready => &self.ready,
            AssetManagementFamilyStatus::Degraded => &self.degraded,
        }
    }
}

impl AssetManagementRecordSetSummary {
    pub fn from_record_sets(
        models: &ModelAssetManagementRecordSet,
        meshes: &MeshAssetManagementRecordSet,
        scenes: &SceneAssetManagementRecordSet,
        scene_entities: &SceneEntityManagementRecordSet,
        material_assets: &MaterialAssetManagementRecordSet,
        materials: &RenderMaterialManagementRecordSet,
        shaders: &ShaderAssetManagementRecordSet,
    ) -> Self {
        let material_degraded_count = material_assets.summary.degraded_count();
        let prepared_material_degraded_count = materials.summary.degraded_count();
        Self {
            managed_record_count: models.summary.model_count
                + meshes.summary.mesh_count
                + scenes.summary.scene_count
                + scene_entities.summary.entity_count
                + material_assets.summary.material_count
                + shaders.summary.shader_count,
            degraded_record_count: meshes.summary.invalid_mesh_count
                + material_degraded_count
                + shaders.summary.not_ready_count,
            model_count: models.summary.model_count,
            model_mesh_referenced_model_count: models.summary.mesh_referenced_model_count,
            model_mesh_reference_count: models.summary.mesh_reference_count,
            mesh_count: meshes.summary.mesh_count,
            valid_mesh_count: meshes.summary.valid_mesh_count,
            invalid_mesh_count: meshes.summary.invalid_mesh_count,
            scene_count: scenes.summary.scene_count,
            scene_entity_count: scenes.summary.entity_count,
            entity_count: scene_entities.summary.entity_count,
            active_entity_count: scene_entities.summary.active_entity_count,
            root_entity_count: scene_entities.summary.root_entity_count,
            entity_direct_reference_count: scene_entities.summary.direct_reference_count,
            entity_camera_count: scene_entities.summary.camera_count,
            entity_mesh_instance_count: scene_entities.summary.mesh_instance_count,
            entity_direct_mesh_reference_count: scene_entities.summary.direct_mesh_reference_count,
            entity_mesh_primitive_binding_count: scene_entities
                .summary
                .mesh_primitive_binding_count,
            entity_mesh_material_binding_count: scene_entities.summary.mesh_material_binding_count,
            entity_collider_material_binding_count: scene_entities
                .summary
                .collider_material_binding_count,
            entity_light_count: scene_entities.summary.light_count,
            entity_physics_component_count: scene_entities.summary.physics_component_count,
            entity_animation_binding_count: scene_entities.summary.animation_binding_count,
            entity_terrain_count: scene_entities.summary.terrain_count,
            entity_tilemap_count: scene_entities.summary.tilemap_count,
            entity_prefab_instance_count: scene_entities.summary.prefab_instance_count,
            material_count: material_assets.summary.material_count,
            material_ready_count: material_assets.summary.ready_count,
            material_degraded_count,
            material_issue_row_count: material_assets.summary.issue_row_count(),
            material_property_override_count: material_assets.summary.property_override_count,
            material_texture_slot_count: material_assets.summary.texture_slot_count,
            material_texture_reference_count: material_assets.summary.texture_reference_count,
            material_fallback_texture_slot_count: material_assets
                .summary
                .fallback_texture_slot_count,
            material_validation_error_count: material_assets.summary.validation_error_count,
            material_validation_diagnostic_count: material_assets
                .summary
                .validation_diagnostic_count,
            material_direct_reference_count: material_assets.summary.direct_reference_count,
            prepared_material_count: materials.summary.total_count,
            prepared_material_ready_count: materials.summary.ready_count,
            prepared_material_degraded_count,
            prepared_material_issue_row_count: materials.summary.issue_row_count(),
            shader_count: shaders.summary.shader_count,
            shader_ready_count: shaders.summary.ready_count,
            shader_not_ready_count: shaders.summary.not_ready_count,
            shader_validation_diagnostic_count: shaders.summary.validation_diagnostic_count,
        }
    }

    pub fn family_summaries(&self) -> Vec<AssetManagementFamilySummary> {
        vec![
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Model,
                self.model_count,
                self.model_count,
                0,
                0,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Mesh,
                self.mesh_count,
                self.valid_mesh_count,
                self.invalid_mesh_count,
                self.invalid_mesh_count,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Scene,
                self.scene_count,
                self.scene_count,
                0,
                0,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Entity,
                self.entity_count,
                self.entity_count,
                0,
                0,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Material,
                self.material_count,
                self.material_ready_count,
                self.material_degraded_count,
                self.material_issue_row_count,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Shader,
                self.shader_count,
                self.shader_ready_count,
                self.shader_not_ready_count,
                self.shader_validation_diagnostic_count,
            ),
        ]
    }
}

impl AssetManagementRecordSets {
    pub fn from_record_sets(
        models: ModelAssetManagementRecordSet,
        meshes: MeshAssetManagementRecordSet,
        scenes: SceneAssetManagementRecordSet,
        scene_entities: SceneEntityManagementRecordSet,
        material_assets: MaterialAssetManagementRecordSet,
        materials: RenderMaterialManagementRecordSet,
        shaders: ShaderAssetManagementRecordSet,
    ) -> Self {
        let summary = AssetManagementRecordSetSummary::from_record_sets(
            &models,
            &meshes,
            &scenes,
            &scene_entities,
            &material_assets,
            &materials,
            &shaders,
        );
        let families = summary.family_summaries();
        let family_status_index = AssetManagementFamilyStatusIndex::from_families(&families);
        Self {
            summary,
            families,
            family_status_index,
            models,
            meshes,
            scenes,
            scene_entities,
            material_assets,
            materials,
            shaders,
        }
    }

    pub fn family_summaries(&self) -> &[AssetManagementFamilySummary] {
        &self.families
    }

    pub fn family_status_index(&self) -> &AssetManagementFamilyStatusIndex {
        &self.family_status_index
    }

    pub fn overview(&self) -> AssetManagementOverview {
        AssetManagementOverview {
            summary: self.summary.clone(),
            families: self.families.clone(),
            family_status_index: self.family_status_index.clone(),
        }
    }
}
