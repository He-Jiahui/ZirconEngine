use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AssetManagementFamilyIssueBucket, AssetManagementFamilyIssueIndex,
    AssetManagementFamilyIssueView, AssetManagementFamilyStatus, AssetManagementFamilyStatusIndex,
    AssetManagementFamilyStatusView, AssetManagementFamilySummary, AssetManagementOverview,
    AssetManagementRecordSets, MaterialAssetManagementRecord, MaterialAssetManagementRecordSet,
    MaterialAssetOverview, MeshAsset, MeshAssetManagementRecord, MeshAssetManagementRecordSet,
    MeshAssetOverview, MeshValidationError, ModelAssetManagementRecord,
    ModelAssetManagementRecordSet, ModelAssetOverview, SceneAssetManagementRecord,
    SceneAssetManagementRecordSet, SceneAssetOverview, SceneEntityManagementRecord,
    SceneEntityManagementRecordSet, ShaderAssetManagementRecord, ShaderAssetManagementRecordSet,
    ShaderAssetReadinessSummary, ShaderReadinessReport, TextureAsset,
};
use std::sync::Arc;

use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
    RenderColorLookupTextureLayout, RenderMaterialAlphaMode, RenderMaterialIssueState,
    RenderMaterialLightingModel, RenderMaterialManagementIssueIndex,
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
    RenderMaterialReadinessReport, RenderMaterialReadinessStatus, RenderMaterialReadinessSummary,
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary, ShadingModelId,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::super::{
    GpuMaterialUniformResource, GpuMeshResource, GpuModelResource, GpuTextureResource,
    MaterialCaptureSeed, MaterialRuntime, OutputTargetTextureResource,
};
use super::ResourceStreamer;
use crate::graphics::material::builtin_shading_model_registry;

impl ResourceStreamer {
    pub(crate) fn asset_manager(&self) -> Arc<ProjectAssetManager> {
        self.asset_manager.clone()
    }

    pub(crate) fn model(&self, id: &ResourceId) -> Option<&Arc<GpuModelResource>> {
        self.models.get(id).map(|prepared| &prepared.resource)
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

    #[allow(dead_code)]
    pub(crate) fn model_asset_overview(&self, id: &ResourceId) -> Option<ModelAssetOverview> {
        self.load_model_asset(*id).map(|asset| asset.overview())
    }

    #[allow(dead_code)]
    pub(crate) fn model_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<ModelAssetManagementRecord> {
        self.asset_manager.model_asset_management_record(*id)
    }

    #[allow(dead_code)]
    pub(crate) fn model_asset_management_records(&self) -> Vec<ModelAssetManagementRecord> {
        self.asset_manager.model_asset_management_records()
    }

    #[allow(dead_code)]
    pub(crate) fn model_asset_management_record_set(&self) -> ModelAssetManagementRecordSet {
        self.asset_manager.model_asset_management_record_set()
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub(crate) fn mesh_asset_overview(
        &self,
        id: &ResourceId,
    ) -> Option<Result<MeshAssetOverview, MeshValidationError>> {
        self.asset_manager
            .load_mesh_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[allow(dead_code)]
    pub(crate) fn mesh_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<Result<MeshAssetManagementRecord, MeshValidationError>> {
        self.asset_manager.mesh_asset_management_record(*id)
    }

    #[allow(dead_code)]
    pub(crate) fn mesh_asset_management_record_results(
        &self,
    ) -> Vec<(
        ResourceId,
        Result<MeshAssetManagementRecord, MeshValidationError>,
    )> {
        self.asset_manager.mesh_asset_management_record_results()
    }

    #[allow(dead_code)]
    pub(crate) fn mesh_asset_management_record_set(&self) -> MeshAssetManagementRecordSet {
        self.asset_manager.mesh_asset_management_record_set()
    }

    #[allow(dead_code)]
    pub(crate) fn scene_asset_overview(&self, id: &ResourceId) -> Option<SceneAssetOverview> {
        self.asset_manager
            .load_scene_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[allow(dead_code)]
    pub(crate) fn scene_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<SceneAssetManagementRecord> {
        self.asset_manager.scene_asset_management_record(*id)
    }

    #[allow(dead_code)]
    pub(crate) fn scene_asset_management_records(&self) -> Vec<SceneAssetManagementRecord> {
        self.asset_manager.scene_asset_management_records()
    }

    #[allow(dead_code)]
    pub(crate) fn scene_asset_management_record_set(&self) -> SceneAssetManagementRecordSet {
        self.asset_manager.scene_asset_management_record_set()
    }

    #[allow(dead_code)]
    pub(crate) fn scene_entity_management_records(&self) -> Vec<SceneEntityManagementRecord> {
        self.asset_manager.scene_entity_management_records()
    }

    #[allow(dead_code)]
    pub(crate) fn scene_entity_management_record_set(&self) -> SceneEntityManagementRecordSet {
        self.asset_manager.scene_entity_management_record_set()
    }

    #[allow(dead_code)]
    pub(crate) fn material_asset_overview(&self, id: &ResourceId) -> Option<MaterialAssetOverview> {
        self.asset_manager
            .load_material_asset(*id)
            .ok()
            .map(|asset| asset.overview())
    }

    #[allow(dead_code)]
    pub(crate) fn material_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<MaterialAssetManagementRecord> {
        self.asset_manager.material_asset_management_record(*id)
    }

    #[allow(dead_code)]
    pub(crate) fn material_asset_management_records(&self) -> Vec<MaterialAssetManagementRecord> {
        self.asset_manager.material_asset_management_records()
    }

    #[allow(dead_code)]
    pub(crate) fn material_asset_management_record_set(&self) -> MaterialAssetManagementRecordSet {
        self.asset_manager.material_asset_management_record_set()
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_readiness_report(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderReadinessReport> {
        self.asset_manager.shader_asset_readiness_report(*id)
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_readiness_summary(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetReadinessSummary> {
        self.asset_manager.shader_asset_readiness_summary(*id)
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetManagementRecord> {
        self.asset_manager.shader_asset_management_record(*id)
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_management_records(&self) -> Vec<ShaderAssetManagementRecord> {
        self.asset_manager.shader_asset_management_records()
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_management_record_set(&self) -> ShaderAssetManagementRecordSet {
        self.asset_manager.shader_asset_management_record_set()
    }

    #[allow(dead_code)]
    pub(crate) fn asset_management_record_sets(&self) -> AssetManagementRecordSets {
        self.asset_manager
            .asset_management_record_sets_with_prepared_materials(
                self.material_management_record_set(),
            )
    }

    #[allow(dead_code)]
    pub(crate) fn asset_management_overview(&self) -> AssetManagementOverview {
        self.asset_management_record_sets().overview()
    }

    #[allow(dead_code)]
    pub(crate) fn asset_management_family_summaries(&self) -> Vec<AssetManagementFamilySummary> {
        self.asset_management_record_sets().families
    }

    #[allow(dead_code)]
    pub(crate) fn asset_management_family_status_index(&self) -> AssetManagementFamilyStatusIndex {
        self.asset_management_record_sets().family_status_index
    }

    #[allow(dead_code)]
    pub(crate) fn asset_management_family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        self.asset_management_record_sets()
            .family_status_view(status)
    }

    #[allow(dead_code)]
    pub(crate) fn asset_management_family_issue_index(&self) -> AssetManagementFamilyIssueIndex {
        self.asset_management_record_sets().family_issue_index
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub(crate) fn material_uniform_payload_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.payload_byte_len)
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_buffer_byte_len(&self, id: &ResourceId) -> Option<u64> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.buffer_byte_len)
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_field_count(&self, id: &ResourceId) -> Option<usize> {
        self.materials.get(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .layout
                .len()
        })
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_unsupported_count(&self, id: &ResourceId) -> Option<usize> {
        self.materials.get(id).map(|prepared| {
            prepared
                .runtime
                .shader_property_uniform_payload
                .unsupported
                .len()
        })
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPropertyUniformSummary> {
        self.materials
            .get(id)
            .map(|prepared| prepared.runtime.shader_property_uniform_payload.summary())
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_fields(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyUniformField>> {
        self.material_readiness_report(id)
            .map(|report| report.uniform_fields.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn material_uniform_unsupported(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyUniformUnsupported>> {
        self.material_readiness_report(id)
            .map(|report| report.uniform_unsupported.clone())
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub(crate) fn material_property_value_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialPropertyValueState>> {
        self.material_readiness_report(id)
            .map(|report| report.property_value_states.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn material_standard_texture_slot_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialTextureSlotSummary> {
        self.material_readiness_report(id)
            .and_then(|report| report.standard_texture_slot_summary)
    }

    #[allow(dead_code)]
    pub(crate) fn material_standard_texture_slot_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialTextureSlotState>> {
        self.material_readiness_report(id)
            .map(|report| report.standard_texture_slot_states.clone())
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub(crate) fn material_texture_slot_states(
        &self,
        id: &ResourceId,
    ) -> Option<Vec<RenderMaterialTextureSlotState>> {
        self.material_readiness_report(id)
            .map(|report| report.non_standard_texture_slot_states.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn material_readiness_report(
        &self,
        id: &ResourceId,
    ) -> Option<&RenderMaterialReadinessReport> {
        self.material(id).map(|material| &material.readiness_report)
    }

    #[allow(dead_code)]
    pub(crate) fn material_readiness_summary(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialReadinessSummary> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::summary)
    }

    #[allow(dead_code)]
    pub(crate) fn material_readiness_status(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialReadinessStatus> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::status)
    }

    #[allow(dead_code)]
    pub(crate) fn material_issue_state(&self, id: &ResourceId) -> Option<RenderMaterialIssueState> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::issue_state)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_snapshot(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialManagementSnapshot> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::management_snapshot)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialManagementRecord> {
        self.material_readiness_report(id)
            .map(|report| report.management_record(*id))
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_records(&self) -> Vec<RenderMaterialManagementRecord> {
        let mut records = self
            .materials
            .iter()
            .map(|(id, prepared)| prepared.runtime.readiness_report.management_record(*id))
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.material_id);
        records
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_record_set(&self) -> RenderMaterialManagementRecordSet {
        RenderMaterialManagementRecordSet::from_records(self.material_management_records())
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_record_set_sorted(
        &self,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementRecordSet {
        RenderMaterialManagementRecordSet::from_sorted_records(
            self.material_management_records(),
            sort_order,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_overview(&self) -> RenderMaterialManagementOverview {
        self.material_management_record_set().overview()
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_overview_sorted(
        &self,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementOverview {
        self.material_management_record_set_sorted(sort_order)
            .overview()
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_query(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryResult {
        self.material_management_record_set().query(query)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_query_selection(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQuerySelection {
        self.material_management_record_set().query_selection(query)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_selection(
        &self,
        material_ids: impl IntoIterator<Item = ResourceId>,
    ) -> RenderMaterialManagementSelection {
        self.material_management_record_set().select(material_ids)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_status_index(&self) -> RenderMaterialManagementStatusIndex {
        self.material_management_record_set().status_index
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_issue_index(&self) -> RenderMaterialManagementIssueIndex {
        self.material_management_record_set().issue_index
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_issue_view(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> RenderMaterialManagementIssueView {
        self.material_management_record_set().issue_view(issue_kind)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_issue_view_sorted(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementIssueView {
        self.material_management_record_set_sorted(sort_order)
            .issue_view(issue_kind)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_status_view(
        &self,
        status: RenderMaterialReadinessStatus,
    ) -> RenderMaterialManagementStatusView {
        self.material_management_record_set().status_view(status)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_status_view_sorted(
        &self,
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementStatusView {
        self.material_management_record_set_sorted(sort_order)
            .status_view(status)
    }

    #[allow(dead_code)]
    pub(crate) fn material_management_record_summary(
        &self,
    ) -> RenderMaterialManagementRecordSummary {
        self.material_management_record_set().summary
    }

    #[allow(dead_code)]
    pub(crate) fn material_prepared_state(
        &self,
        id: &ResourceId,
    ) -> Option<RenderMaterialPreparedState> {
        self.material_readiness_report(id)
            .map(RenderMaterialReadinessReport::prepared_state)
    }

    #[allow(dead_code)]
    pub(crate) fn material_capture_seed(&self, id: &ResourceId) -> Option<MaterialCaptureSeed> {
        self.material(id)
            .map(|material| material.capture_seed())
            .or_else(|| {
                self.asset_manager
                    .load_material_asset(*id)
                    .ok()
                    .map(|material| {
                        let descriptor = material.standard_material_descriptor();
                        let lighting_model = if descriptor.unlit {
                            RenderMaterialLightingModel::Unlit
                        } else {
                            descriptor.lighting_model.clone()
                        };
                        let shading_model_id = shading_model_id_for_lighting_model(&lighting_model);
                        MaterialCaptureSeed {
                            base_color: Vec4::from_array(descriptor.base_color),
                            emissive: Vec3::from_array(descriptor.emissive),
                            metallic: descriptor.metallic,
                            roughness: descriptor.roughness,
                            double_sided: descriptor.double_sided,
                            alpha_blend: matches!(
                                descriptor.alpha_mode,
                                RenderMaterialAlphaMode::Blend
                            ),
                            alpha_cutoff: match descriptor.alpha_mode {
                                RenderMaterialAlphaMode::Mask { cutoff } => Some(cutoff),
                                _ => None,
                            },
                            lighting_model,
                            shading_model_id,
                            unlit: descriptor.unlit || descriptor.lighting_model.is_unlit(),
                            cast_shadows: descriptor.cast_shadows,
                            receive_shadows: descriptor.receive_shadows,
                            taa_reactive_mask_strength: descriptor.taa_reactive_mask_strength,
                            base_color_texture: self
                                .resolve_texture_reference(
                                    "base_color_texture",
                                    descriptor.base_color_texture.as_ref(),
                                )
                                .id(),
                            base_color_texture_transform: descriptor.base_color_texture_transform,
                            base_color_texture_uv_channel: descriptor.base_color_texture_uv_channel,
                            normal_texture: self
                                .resolve_texture_reference(
                                    "normal_texture",
                                    descriptor.normal_texture.as_ref(),
                                )
                                .id(),
                            normal_texture_transform: descriptor.normal_texture_transform,
                            normal_texture_uv_channel: descriptor.normal_texture_uv_channel,
                            metallic_roughness_texture: self
                                .resolve_texture_reference(
                                    "metallic_roughness_texture",
                                    descriptor.metallic_roughness_texture.as_ref(),
                                )
                                .id(),
                            metallic_roughness_texture_transform: descriptor
                                .metallic_roughness_texture_transform,
                            metallic_roughness_texture_uv_channel: descriptor
                                .metallic_roughness_texture_uv_channel,
                            occlusion_texture: self
                                .resolve_texture_reference(
                                    "occlusion_texture",
                                    descriptor.occlusion_texture.as_ref(),
                                )
                                .id(),
                            occlusion_texture_transform: descriptor.occlusion_texture_transform,
                            occlusion_texture_uv_channel: descriptor.occlusion_texture_uv_channel,
                            emissive_texture: self
                                .resolve_texture_reference(
                                    "emissive_texture",
                                    descriptor.emissive_texture.as_ref(),
                                )
                                .id(),
                            emissive_texture_transform: descriptor.emissive_texture_transform,
                            emissive_texture_uv_channel: descriptor.emissive_texture_uv_channel,
                        }
                    })
            })
    }

    #[allow(dead_code)]
    pub(crate) fn sample_texture_rgba(&self, id: Option<ResourceId>, uv: [f32; 2]) -> Option<Vec4> {
        id.and_then(|texture_id| {
            self.asset_manager
                .load_texture_asset(texture_id)
                .ok()
                .and_then(|texture| sample_texture_asset_rgba(&texture, uv))
        })
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
                    .then_some(&prepared.resource.view)
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
            .map(|prepared| prepared.resource.clone())
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

fn shading_model_id_for_lighting_model(model: &RenderMaterialLightingModel) -> ShadingModelId {
    builtin_shading_model_registry()
        .resolve_lighting_model(model)
        .map(|descriptor| descriptor.id)
        .unwrap_or(SHADING_MODEL_ID_STANDARD_PBR)
}

#[allow(dead_code)]
fn sample_texture_asset_rgba(texture: &TextureAsset, uv: [f32; 2]) -> Option<Vec4> {
    if texture.width == 0 || texture.height == 0 {
        return None;
    }

    let u = wrap01(uv[0]);
    let v = wrap01(uv[1]);
    let x = ((texture.width - 1) as f32 * u).round() as usize;
    let y = ((texture.height - 1) as f32 * v).round() as usize;
    let index = ((y * texture.width as usize) + x) * 4;
    let rgba = texture.rgba.get(index..index + 4)?;
    Some(Vec4::new(
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
        rgba[3] as f32 / 255.0,
    ))
}

#[allow(dead_code)]
fn wrap01(value: f32) -> f32 {
    let wrapped = value.fract();
    if wrapped < 0.0 {
        wrapped + 1.0
    } else {
        wrapped
    }
}
