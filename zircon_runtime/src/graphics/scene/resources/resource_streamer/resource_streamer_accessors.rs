use crate::asset::MeshAsset;
use crate::asset::pipeline::manager::ProjectAssetManager;
#[cfg(test)]
use crate::asset::{
    AssetManagementFamilyIssueBucket, AssetManagementFamilyIssueIndex,
    AssetManagementFamilyIssueView, AssetManagementFamilyStatus, AssetManagementFamilyStatusIndex,
    AssetManagementFamilyStatusView, AssetManagementFamilySummary, AssetManagementOverview,
    AssetManagementRecordSets, MaterialAssetManagementRecord, MaterialAssetManagementRecordSet,
    MaterialAssetOverview, MeshAssetManagementRecord, MeshAssetManagementRecordSet,
    MeshAssetOverview, MeshValidationError, ModelAssetManagementRecord,
    ModelAssetManagementRecordSet, ModelAssetOverview, SceneAssetManagementRecord,
    SceneAssetManagementRecordSet, SceneAssetOverview, SceneEntityManagementRecord,
    SceneEntityManagementRecordSet, ShaderAssetManagementRecord, ShaderAssetManagementRecordSet,
    ShaderAssetReadinessSummary, ShaderReadinessReport,
};
use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::render::{
    MaterialPropertyOverrideBlock, RenderCameraTargetGraphImportReport,
    RenderCameraTargetWritebackReport, RenderColorLookupTextureLayout,
    RenderMaterialPropertyUniformPayload, RenderMaterialReadinessReport,
    RenderMaterialReadinessSummary, RenderShaderDefinitionValue,
};
use crate::core::resource::ResourceId;
use crate::graphics::GraphicsError;
use crate::graphics::shader::ShaderTemplateInclude;

mod material_capture;
#[cfg(test)]
mod material_diagnostics;

use super::super::{
    GpuMaterialUniformResource, GpuMeshResource, GpuModelResource, GpuTextureResource,
    MaterialRuntime, OutputTargetTextureResource,
};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn asset_manager(&self) -> Result<Arc<ProjectAssetManager>, GraphicsError> {
        self.asset_manager_access
            .resolve()
            .map_err(|error| GraphicsError::Asset(error.to_string()))
    }

    #[cfg(test)]
    fn test_asset_manager(&self) -> Arc<ProjectAssetManager> {
        self.asset_manager()
            .expect("test ProjectAssetManager runtime must remain available")
    }

    pub(crate) fn model(&self, id: &ResourceId) -> Option<&Arc<GpuModelResource>> {
        self.models.get(id).map(|prepared| {
            debug_assert_eq!(
                prepared.resource.id(),
                *id,
                "GpuModelResource identity must match the ResourceStreamer model key",
            );
            &prepared.resource
        })
    }

    pub(crate) fn model_revision(&self, id: &ResourceId) -> Option<u64> {
        self.models.get(id).map(|prepared| prepared.revision)
    }

    pub(crate) fn mesh(&self, id: &ResourceId) -> Option<&Arc<GpuMeshResource>> {
        self.meshes.get(id).map(|prepared| &prepared.resource)
    }

    pub(crate) fn mesh_revision(&self, id: &ResourceId) -> Option<u64> {
        self.meshes.get(id).map(|prepared| prepared.revision)
    }

    pub(crate) fn mesh_asset(&self, id: &ResourceId) -> Option<&Arc<MeshAsset>> {
        self.meshes.get(id).map(|prepared| &prepared.asset)
    }

    #[cfg(test)]
    pub(crate) fn model_asset_overview(&self, id: &ResourceId) -> Option<ModelAssetOverview> {
        self.load_model_asset(*id).map(|asset| asset.overview())
    }

    #[cfg(test)]
    pub(crate) fn model_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<ModelAssetManagementRecord> {
        self.test_asset_manager().model_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn model_asset_management_records(&self) -> Vec<ModelAssetManagementRecord> {
        self.test_asset_manager().model_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn model_asset_management_record_set(&self) -> ModelAssetManagementRecordSet {
        self.test_asset_manager()
            .model_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn prepared_model_asset_management_records(
        &self,
    ) -> Vec<ModelAssetManagementRecord> {
        let mut records = self
            .models
            .iter()
            .map(|(id, prepared)| prepared.asset.management_record(*id))
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.model_id);
        records
    }

    #[cfg(test)]
    pub(crate) fn mesh_asset_overview(
        &self,
        id: &ResourceId,
    ) -> Option<Result<MeshAssetOverview, MeshValidationError>> {
        self.test_asset_manager()
            .load_mesh_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[cfg(test)]
    pub(crate) fn mesh_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<Result<MeshAssetManagementRecord, MeshValidationError>> {
        self.test_asset_manager().mesh_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn mesh_asset_management_record_results(
        &self,
    ) -> Vec<(
        ResourceId,
        Result<MeshAssetManagementRecord, MeshValidationError>,
    )> {
        self.test_asset_manager()
            .mesh_asset_management_record_results()
    }

    #[cfg(test)]
    pub(crate) fn mesh_asset_management_record_set(&self) -> MeshAssetManagementRecordSet {
        self.test_asset_manager().mesh_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_overview(&self, id: &ResourceId) -> Option<SceneAssetOverview> {
        self.test_asset_manager()
            .load_scene_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<SceneAssetManagementRecord> {
        self.test_asset_manager().scene_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_management_records(&self) -> Vec<SceneAssetManagementRecord> {
        self.test_asset_manager().scene_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_management_record_set(&self) -> SceneAssetManagementRecordSet {
        self.test_asset_manager()
            .scene_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn scene_entity_management_records(&self) -> Vec<SceneEntityManagementRecord> {
        self.test_asset_manager().scene_entity_management_records()
    }

    #[cfg(test)]
    pub(crate) fn scene_entity_management_record_set(&self) -> SceneEntityManagementRecordSet {
        self.test_asset_manager()
            .scene_entity_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn material_asset_overview(&self, id: &ResourceId) -> Option<MaterialAssetOverview> {
        self.test_asset_manager()
            .load_material_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[cfg(test)]
    pub(crate) fn material_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<MaterialAssetManagementRecord> {
        self.test_asset_manager()
            .material_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn material_asset_management_records(&self) -> Vec<MaterialAssetManagementRecord> {
        self.test_asset_manager()
            .material_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn material_asset_management_record_set(&self) -> MaterialAssetManagementRecordSet {
        self.test_asset_manager()
            .material_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_readiness_report(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderReadinessReport> {
        self.test_asset_manager().shader_asset_readiness_report(*id)
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_readiness_summary(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetReadinessSummary> {
        self.test_asset_manager()
            .shader_asset_readiness_summary(*id)
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetManagementRecord> {
        self.test_asset_manager()
            .shader_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_management_records(&self) -> Vec<ShaderAssetManagementRecord> {
        self.test_asset_manager().shader_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_management_record_set(&self) -> ShaderAssetManagementRecordSet {
        self.test_asset_manager()
            .shader_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn asset_management_record_sets(&self) -> AssetManagementRecordSets {
        self.test_asset_manager()
            .asset_management_record_sets_with_prepared_materials(
                self.material_management_record_set(),
            )
    }

    #[cfg(test)]
    pub(crate) fn asset_management_overview(&self) -> AssetManagementOverview {
        self.asset_management_record_sets().overview()
    }

    #[cfg(test)]
    pub(crate) fn asset_management_family_summaries(&self) -> Vec<AssetManagementFamilySummary> {
        self.asset_management_record_sets().families
    }

    #[cfg(test)]
    pub(crate) fn asset_management_family_status_index(&self) -> AssetManagementFamilyStatusIndex {
        self.asset_management_record_sets().family_status_index
    }

    #[cfg(test)]
    pub(crate) fn asset_management_family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        self.asset_management_record_sets()
            .family_status_view(status)
    }

    #[cfg(test)]
    pub(crate) fn asset_management_family_issue_index(&self) -> AssetManagementFamilyIssueIndex {
        self.asset_management_record_sets().family_issue_index
    }

    #[cfg(test)]
    pub(crate) fn asset_management_family_issue_view(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> AssetManagementFamilyIssueView {
        self.asset_management_record_sets()
            .family_issue_view(bucket)
    }

    pub(crate) fn material(&self, id: &ResourceId) -> Option<&MaterialRuntime> {
        self.materials.get(id).map(|prepared| &prepared.runtime)
    }

    pub(crate) fn material_revision(&self, id: &ResourceId) -> Option<u64> {
        self.materials
            .get(id)
            .and_then(|prepared| prepared.revision)
    }

    pub(crate) fn material_uniform(&self, id: &ResourceId) -> Arc<GpuMaterialUniformResource> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.clone())
            .unwrap_or_else(|| self.fallback_material_uniform.clone())
    }

    pub(crate) fn material_uniform_payload_with_overrides(
        &self,
        id: &ResourceId,
        overrides: &MaterialPropertyOverrideBlock,
    ) -> Option<RenderMaterialPropertyUniformPayload> {
        if overrides.is_empty() {
            return None;
        }
        self.materials.get(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .with_override_block(overrides)
        })
    }

    pub(crate) fn standard_material_uniform(
        &self,
        id: &ResourceId,
    ) -> Arc<GpuMaterialUniformResource> {
        self.materials
            .get(id)
            .map(|prepared| prepared.standard_uniform.clone())
            .unwrap_or_else(|| self.fallback_standard_material_uniform.clone())
    }

    pub(crate) fn material_readiness_report(
        &self,
        id: &ResourceId,
    ) -> Option<&RenderMaterialReadinessReport> {
        self.material(id).map(|material| &material.readiness_report)
    }

    pub(crate) fn material_readiness_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialReadinessSummary> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::summary)
    }

    pub(crate) fn texture(&self, id: Option<ResourceId>) -> Arc<GpuTextureResource> {
        id.and_then(|texture_id| {
            self.textures
                .get(&texture_id)
                .map(|prepared| prepared.resource.clone())
        })
        .unwrap_or_else(|| self.fallback_texture.clone())
    }

    pub(crate) fn normal_texture(&self, id: Option<ResourceId>) -> Arc<GpuTextureResource> {
        id.and_then(|texture_id| {
            self.textures
                .get(&texture_id)
                .map(|prepared| prepared.resource.clone())
        })
        .unwrap_or_else(|| self.fallback_normal_texture.clone())
    }

    pub(crate) fn prepared_post_process_lut_2d_view(
        &self,
        id: ResourceId,
        layout: RenderColorLookupTextureLayout,
    ) -> Option<(&wgpu::TextureView, bool)> {
        self.textures.get(&id).and_then(|prepared| {
            let descriptor = &prepared.resource.descriptor;
            layout
                .accepts_current_post_process_binding(descriptor)
                .then_some((
                    &prepared.resource.view,
                    layout.matches_texture_2d_strip(descriptor),
                ))
        })
    }

    pub(crate) fn prepared_post_process_lut_3d_view(
        &self,
        id: ResourceId,
        layout: RenderColorLookupTextureLayout,
    ) -> Option<&wgpu::TextureView> {
        self.post_process_lut_textures
            .get(&id)
            .and_then(|prepared| {
                layout
                    .matches_texture_3d(&prepared.resource.descriptor)
                    .then_some(prepared.resource.view())
            })
    }

    pub(crate) fn shader_source(&self, shader_id: &ResourceId) -> Option<&str> {
        self.shaders
            .get(shader_id)
            .map(|shader| shader.runtime.source.as_str())
    }

    pub(crate) fn shader_generated_material_source(&self, shader_id: &ResourceId) -> Option<&str> {
        self.shaders
            .get(shader_id)
            .map(|shader| shader.runtime.generated_material_wgsl.as_str())
            .filter(|source| !source.trim().is_empty())
    }

    pub(crate) fn shader_import_path(&self, shader_id: &ResourceId) -> Option<&str> {
        self.shaders
            .get(shader_id)
            .and_then(|shader| shader.runtime.import_path.as_deref())
    }

    pub(crate) fn shader_is_surface(&self, shader_id: &ResourceId) -> bool {
        self.shaders
            .get(shader_id)
            .is_some_and(|shader| shader.runtime.kind.participates_in_material_variants())
    }

    pub(crate) fn shader_uses_material_surface_source(&self, shader_id: &ResourceId) -> bool {
        self.shaders.get(shader_id).is_some_and(|shader| {
            shader.runtime.kind.participates_in_material_variants()
                && shader.runtime.source.contains("fn zr_material_surface")
        })
    }

    pub(crate) fn shader_module_include_sources(
        &self,
        shader_id: &ResourceId,
    ) -> Vec<ShaderTemplateInclude> {
        let Ok(asset_manager) = self.asset_manager() else {
            return Vec::new();
        };
        let mut includes = Vec::new();
        let mut visited = HashSet::new();
        self.collect_shader_module_include_sources(
            asset_manager.as_ref(),
            shader_id,
            &mut visited,
            &mut includes,
        );
        includes
    }

    fn collect_shader_module_include_sources(
        &self,
        asset_manager: &ProjectAssetManager,
        shader_id: &ResourceId,
        visited: &mut HashSet<ResourceId>,
        includes: &mut Vec<ShaderTemplateInclude>,
    ) {
        if !visited.insert(*shader_id) {
            return;
        }
        let Some(shader) = self.shaders.get(shader_id) else {
            return;
        };
        for import in &shader.runtime.imports {
            let Some(reference) = import.redirect.as_ref() else {
                continue;
            };
            let Some(import_id) = asset_manager.resolve_asset_id(&reference.locator) else {
                continue;
            };
            if let Some(import_shader) = self.shaders.get(&import_id) {
                if import_shader.runtime.kind.is_include() {
                    if let Some(import_path) = import_shader.runtime.import_path.as_ref() {
                        includes.push(ShaderTemplateInclude::new(
                            import_path.clone(),
                            import_shader.runtime.source.clone(),
                        ));
                    }
                }
            }
            self.collect_shader_module_include_sources(
                asset_manager,
                &import_id,
                visited,
                includes,
            );
        }
    }

    pub(crate) fn shader_material_option_defines(
        &self,
        shader_id: &ResourceId,
        material_option_bits: u32,
    ) -> Vec<RenderShaderDefinitionValue> {
        self.shaders
            .get(shader_id)
            .map(|shader| {
                shader
                    .runtime
                    .material_option_table
                    .definition_values_for_bits(material_option_bits)
            })
            .unwrap_or_default()
    }

    pub(crate) fn last_material_count(&self) -> usize {
        self.last_material_count
    }

    pub(crate) fn last_material_ready_count(&self) -> usize {
        self.last_material_ready_count
    }

    pub(crate) fn last_material_fallback_count(&self) -> usize {
        self.last_material_fallback_count
    }

    pub(crate) fn last_material_validation_error_count(&self) -> usize {
        self.last_material_validation_error_count
    }

    pub(crate) fn last_material_diagnostic_count(&self) -> usize {
        self.last_material_diagnostic_count
    }

    pub(crate) fn last_sprite_count(&self) -> usize {
        self.last_sprite_count
    }

    pub(crate) fn last_sprite_ready_count(&self) -> usize {
        self.last_sprite_ready_count
    }

    pub(crate) fn last_sprite_texture_fallback_count(&self) -> usize {
        self.last_sprite_texture_fallback_count
    }

    pub(crate) fn last_post_process_lut_request_count(&self) -> usize {
        self.last_post_process_lut_request_count
    }

    pub(crate) fn last_post_process_lut_ready_count(&self) -> usize {
        self.last_post_process_lut_ready_count
    }

    pub(crate) fn last_post_process_lut_fallback_count(&self) -> usize {
        self.last_post_process_lut_fallback_count
    }

    pub(crate) fn last_post_process_lut_2d_strip_ready_count(&self) -> usize {
        self.last_post_process_lut_2d_strip_ready_count
    }

    pub(crate) fn last_post_process_lut_3d_request_count(&self) -> usize {
        self.last_post_process_lut_3d_request_count
    }

    pub(crate) fn last_post_process_lut_unsupported_shape_count(&self) -> usize {
        self.last_post_process_lut_unsupported_shape_count
    }

    pub(in crate::graphics::scene) fn output_target_texture_resource(
        &self,
        id: &ResourceId,
    ) -> Option<Arc<OutputTargetTextureResource>> {
        self.output_target_textures
            .get(id)
            .map(|prepared| Arc::clone(prepared.resource()))
    }

    pub(in crate::graphics::scene) fn set_last_output_target_graph_import_report(
        &mut self,
        report: RenderCameraTargetGraphImportReport,
    ) {
        self.last_output_target_graph_import_report = report;
    }

    pub(crate) fn last_output_target_writeback_report(&self) -> RenderCameraTargetWritebackReport {
        self.last_output_target_writeback_report
    }

    pub(crate) fn last_output_target_graph_import_report(
        &self,
    ) -> RenderCameraTargetGraphImportReport {
        self.last_output_target_graph_import_report
    }
}
