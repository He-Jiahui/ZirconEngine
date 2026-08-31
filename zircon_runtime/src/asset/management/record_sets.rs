use crate::asset::assets::{
    MaterialAssetManagementRecordSet, MeshAssetManagementRecordSet, ModelAssetManagementRecordSet,
    SceneAssetManagementRecordSet, SceneEntityManagementRecordSet, ShaderAssetManagementRecordSet,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;

use super::{
    AssetManagementFamilyIssueBucket, AssetManagementFamilyIssueIndex,
    AssetManagementFamilyIssueView, AssetManagementFamilyKind, AssetManagementFamilyStatus,
    AssetManagementFamilyStatusIndex, AssetManagementFamilyStatusView,
    AssetManagementFamilySummary, AssetManagementOverview, AssetManagementRecordSetSummary,
    AssetManagementRecordSets,
};

impl AssetManagementOverview {
    pub fn from_summary(summary: AssetManagementRecordSetSummary) -> Self {
        let families = summary.family_summaries();
        let family_status_index = AssetManagementFamilyStatusIndex::from_families(&families);
        let family_issue_index = AssetManagementFamilyIssueIndex::from_families(&families);
        Self {
            summary,
            families,
            family_status_index,
            family_issue_index,
        }
    }

    pub fn family_summaries(&self) -> &[AssetManagementFamilySummary] {
        &self.families
    }

    pub fn family_status_index(&self) -> &AssetManagementFamilyStatusIndex {
        &self.family_status_index
    }

    pub fn family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        AssetManagementFamilyStatusView::from_families(&self.families, status)
    }

    pub fn family_issue_index(&self) -> &AssetManagementFamilyIssueIndex {
        &self.family_issue_index
    }

    pub fn family_issue_view(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> AssetManagementFamilyIssueView {
        AssetManagementFamilyIssueView::from_families(&self.families, bucket)
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
            mesh_morph_target_count: meshes.summary.morph_target_count,
            mesh_morph_target_attribute_count: meshes.summary.morph_target_attribute_count,
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
            entity_morph_weight_count: scene_entities.summary.morph_weight_count,
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
            shader_issue_row_count: shaders.summary.issue_row_count(),
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
                self.shader_issue_row_count,
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
        let family_issue_index = AssetManagementFamilyIssueIndex::from_families(&families);
        Self {
            summary,
            families,
            family_status_index,
            family_issue_index,
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

    pub fn family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        AssetManagementFamilyStatusView::from_families(&self.families, status)
    }

    pub fn family_issue_index(&self) -> &AssetManagementFamilyIssueIndex {
        &self.family_issue_index
    }

    pub fn family_issue_view(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> AssetManagementFamilyIssueView {
        AssetManagementFamilyIssueView::from_families(&self.families, bucket)
    }

    pub fn overview(&self) -> AssetManagementOverview {
        AssetManagementOverview {
            summary: self.summary.clone(),
            families: self.families.clone(),
            family_status_index: self.family_status_index.clone(),
            family_issue_index: self.family_issue_index.clone(),
        }
    }
}
