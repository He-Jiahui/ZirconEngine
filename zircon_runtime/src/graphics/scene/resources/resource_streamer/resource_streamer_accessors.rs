use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::MeshAsset;
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
use std::sync::Arc;

use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
    RenderColorLookupTextureLayout, RenderMaterialReadinessReport, RenderMaterialReadinessSummary,
};
#[cfg(test)]
use crate::core::framework::render::{
    RenderMaterialIssueState, RenderMaterialManagementIssueIndex,
    RenderMaterialManagementIssueKind, RenderMaterialManagementIssueView,
    RenderMaterialManagementOverview, RenderMaterialManagementQuery,
    RenderMaterialManagementQueryResult, RenderMaterialManagementQuerySelection,
    RenderMaterialManagementRecord, RenderMaterialManagementRecordSet,
    RenderMaterialManagementRecordSummary, RenderMaterialManagementSelection,
    RenderMaterialManagementSnapshot, RenderMaterialManagementSortOrder,
    RenderMaterialManagementStatusIndex, RenderMaterialManagementStatusView,
    RenderMaterialPreparedState, RenderMaterialPropertyUniformField,
    RenderMaterialPropertyUniformSummary, RenderMaterialPropertyUniformUnsupported,
    RenderMaterialPropertyValueState, RenderMaterialPropertyValueSummary,
    RenderMaterialReadinessStatus, RenderMaterialTextureSlotState,
    RenderMaterialTextureSlotSummary,
};
use crate::core::resource::ResourceId;

#[cfg(test)]
mod material_capture;

use super::super::{
    GpuMaterialUniformResource, GpuMeshResource, GpuModelResource, GpuTextureResource,
    MaterialRuntime, OutputTargetTextureResource,
};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn asset_manager(&self) -> Arc<ProjectAssetManager> {
        self.asset_manager.clone()
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
        self.asset_manager.model_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn model_asset_management_records(&self) -> Vec<ModelAssetManagementRecord> {
        self.asset_manager.model_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn model_asset_management_record_set(&self) -> ModelAssetManagementRecordSet {
        self.asset_manager.model_asset_management_record_set()
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
        self.asset_manager
            .load_mesh_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[cfg(test)]
    pub(crate) fn mesh_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<Result<MeshAssetManagementRecord, MeshValidationError>> {
        self.asset_manager.mesh_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn mesh_asset_management_record_results(
        &self,
    ) -> Vec<(
        ResourceId,
        Result<MeshAssetManagementRecord, MeshValidationError>,
    )> {
        self.asset_manager.mesh_asset_management_record_results()
    }

    #[cfg(test)]
    pub(crate) fn mesh_asset_management_record_set(&self) -> MeshAssetManagementRecordSet {
        self.asset_manager.mesh_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_overview(&self, id: &ResourceId) -> Option<SceneAssetOverview> {
        self.asset_manager
            .load_scene_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<SceneAssetManagementRecord> {
        self.asset_manager.scene_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_management_records(&self) -> Vec<SceneAssetManagementRecord> {
        self.asset_manager.scene_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn scene_asset_management_record_set(&self) -> SceneAssetManagementRecordSet {
        self.asset_manager.scene_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn scene_entity_management_records(&self) -> Vec<SceneEntityManagementRecord> {
        self.asset_manager.scene_entity_management_records()
    }

    #[cfg(test)]
    pub(crate) fn scene_entity_management_record_set(&self) -> SceneEntityManagementRecordSet {
        self.asset_manager.scene_entity_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn material_asset_overview(&self, id: &ResourceId) -> Option<MaterialAssetOverview> {
        self.asset_manager
            .load_material_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[cfg(test)]
    pub(crate) fn material_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<MaterialAssetManagementRecord> {
        self.asset_manager.material_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn material_asset_management_records(&self) -> Vec<MaterialAssetManagementRecord> {
        self.asset_manager.material_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn material_asset_management_record_set(&self) -> MaterialAssetManagementRecordSet {
        self.asset_manager.material_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_readiness_report(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderReadinessReport> {
        self.asset_manager.shader_asset_readiness_report(*id)
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_readiness_summary(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetReadinessSummary> {
        self.asset_manager.shader_asset_readiness_summary(*id)
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetManagementRecord> {
        self.asset_manager.shader_asset_management_record(*id)
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_management_records(&self) -> Vec<ShaderAssetManagementRecord> {
        self.asset_manager.shader_asset_management_records()
    }

    #[cfg(test)]
    pub(crate) fn shader_asset_management_record_set(&self) -> ShaderAssetManagementRecordSet {
        self.asset_manager.shader_asset_management_record_set()
    }

    #[cfg(test)]
    pub(crate) fn asset_management_record_sets(&self) -> AssetManagementRecordSets {
        self.asset_manager
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

    pub(crate) fn standard_material_uniform(
        &self,
        id: &ResourceId,
    ) -> Arc<GpuMaterialUniformResource> {
        self.materials
            .get(id)
            .map(|prepared| prepared.standard_uniform.clone())
            .unwrap_or_else(|| self.fallback_standard_material_uniform.clone())
    }

    #[cfg(test)]
    pub(crate) fn material_uniform_payload_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.payload_byte_len())
    }

    #[cfg(test)]
    pub(crate) fn material_uniform_buffer_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.buffer_byte_len())
    }

    #[cfg(test)]
    pub(crate) fn material_uniform_field_count(&self, id: &ResourceId) -> Option<usize> {
        self.materials.get(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .layout
                .len()
        })
    }

    #[cfg(test)]
    pub(crate) fn material_uniform_unsupported_count(&self, id: &ResourceId) -> Option<usize> {
        self.materials.get(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .unsupported
                .len()
        })
    }

    #[cfg(test)]
    pub(crate) fn material_uniform_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPropertyUniformSummary> {
        self.materials
            .get(id)
            .map(|prepared| prepared.runtime.shader_property_uniform_payload.summary())
    }

    #[cfg(test)]
    pub(crate) fn material_uniform_fields(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyUniformField>> {
        self.material_readiness_report(id)
            .map(|report| report.uniform_fields.clone())
    }

    #[cfg(test)]
    pub(crate) fn material_uniform_unsupported(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyUniformUnsupported>> {
        self.material_readiness_report(id)
            .map(|report| report.uniform_unsupported.clone())
    }

    #[cfg(test)]
    pub(crate) fn material_property_value_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPropertyValueSummary> {
        self.materials.get(id).map(|prepared| {
            RenderMaterialPropertyValueSummary::from_values(
                &prepared.runtime.shader_property_values,
            )
        })
    }

    #[cfg(test)]
    pub(crate) fn material_property_value_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyValueState>> {
        self.material_readiness_report(id)
            .map(|report| report.property_value_states.clone())
    }

    #[cfg(test)]
    pub(crate) fn material_standard_texture_slot_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialTextureSlotSummary> {
        self.material_readiness_report(id)
            .and_then(|report| report.standard_texture_slot_summary)
    }

    #[cfg(test)]
    pub(crate) fn material_standard_texture_slot_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialTextureSlotState>> {
        self.material_readiness_report(id)
            .map(|report| report.standard_texture_slot_states.clone())
    }

    #[cfg(test)]
    pub(crate) fn material_texture_slot_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialTextureSlotSummary> {
        self.materials.get(id).map(|prepared| {
            RenderMaterialTextureSlotSummary::from_non_standard_slots(
                &prepared.runtime.non_standard_texture_slots,
            )
        })
    }

    #[cfg(test)]
    pub(crate) fn material_texture_slot_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialTextureSlotState>> {
        self.material_readiness_report(id)
            .map(|report| report.non_standard_texture_slot_states.clone())
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

    #[cfg(test)]
    pub(crate) fn material_readiness_status(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialReadinessStatus> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::status)
    }

    #[cfg(test)]
    pub(crate) fn material_issue_state(&self, id: &ResourceId) -> Option<RenderMaterialIssueState> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::issue_state)
    }

    #[cfg(test)]
    pub(crate) fn material_management_snapshot(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialManagementSnapshot> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::management_snapshot)
    }

    #[cfg(test)]
    pub(crate) fn material_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialManagementRecord> {
        self.material_readiness_report(id)
            .map(|report| report.management_record(*id))
    }

    #[cfg(test)]
    pub(crate) fn material_management_records(&self) -> Vec<RenderMaterialManagementRecord> {
        let mut records = self
            .materials
            .iter()
            .map(|(id, prepared)| prepared.runtime.readiness_report.management_record(*id))
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.material_id);
        records
    }

    #[cfg(test)]
    pub(crate) fn material_management_record_set(&self) -> RenderMaterialManagementRecordSet {
        RenderMaterialManagementRecordSet::from_records(self.material_management_records())
    }

    #[cfg(test)]
    pub(crate) fn material_management_record_set_sorted(
        &self,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementRecordSet {
        RenderMaterialManagementRecordSet::from_sorted_records(
            self.material_management_records(),
            sort_order,
        )
    }

    #[cfg(test)]
    pub(crate) fn material_management_overview(&self) -> RenderMaterialManagementOverview {
        self.material_management_record_set().overview()
    }

    #[cfg(test)]
    pub(crate) fn material_management_overview_sorted(
        &self,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementOverview {
        self.material_management_record_set_sorted(sort_order)
            .overview()
    }

    #[cfg(test)]
    pub(crate) fn material_management_query(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryResult {
        self.material_management_record_set().query(query)
    }

    #[cfg(test)]
    pub(crate) fn material_management_query_selection(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQuerySelection {
        self.material_management_record_set().query_selection(query)
    }

    #[cfg(test)]
    pub(crate) fn material_management_selection(
        &self,
        material_ids: impl IntoIterator<Item = ResourceId>,
    ) -> RenderMaterialManagementSelection {
        self.material_management_record_set().select(material_ids)
    }

    #[cfg(test)]
    pub(crate) fn material_management_status_index(&self) -> RenderMaterialManagementStatusIndex {
        self.material_management_record_set().status_index
    }

    #[cfg(test)]
    pub(crate) fn material_management_issue_index(&self) -> RenderMaterialManagementIssueIndex {
        self.material_management_record_set().issue_index
    }

    #[cfg(test)]
    pub(crate) fn material_management_issue_view(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> RenderMaterialManagementIssueView {
        self.material_management_record_set().issue_view(issue_kind)
    }

    #[cfg(test)]
    pub(crate) fn material_management_issue_view_sorted(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementIssueView {
        self.material_management_record_set_sorted(sort_order)
            .issue_view(issue_kind)
    }

    #[cfg(test)]
    pub(crate) fn material_management_status_view(
        &self,
        status: RenderMaterialReadinessStatus,
    ) -> RenderMaterialManagementStatusView {
        self.material_management_record_set().status_view(status)
    }

    #[cfg(test)]
    pub(crate) fn material_management_status_view_sorted(
        &self,
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementStatusView {
        self.material_management_record_set_sorted(sort_order)
            .status_view(status)
    }

    #[cfg(test)]
    pub(crate) fn material_management_record_summary(
        &self,
    ) -> RenderMaterialManagementRecordSummary {
        self.material_management_record_set().summary
    }

    #[cfg(test)]
    pub(crate) fn material_prepared_state(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPreparedState> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::prepared_state)
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
