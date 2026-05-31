use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AssetManagementFamilyStatusIndex, AssetManagementFamilySummary, AssetManagementOverview,
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
    RenderMaterialAlphaMode, RenderMaterialIssueState, RenderMaterialManagementIssueIndex,
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
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::{ResourceId, ResourceKind};

use super::super::{
    GpuMaterialUniformResource, GpuMeshResource, GpuModelResource, GpuTextureResource,
    MaterialCaptureSeed, MaterialRuntime,
};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn asset_manager(&self) -> Arc<ProjectAssetManager> {
        self.asset_manager.clone()
    }

    fn asset_ids_by_kind(&self, kind: ResourceKind) -> Vec<ResourceId> {
        let mut ids = {
            let resource_manager = self.asset_manager.resource_manager();
            let registry = resource_manager.registry();
            registry
                .values()
                .filter(|record| record.kind == kind)
                .map(|record| record.id())
                .collect::<Vec<_>>()
        };
        ids.sort();
        ids
    }

    pub(crate) fn model(&self, id: &ResourceId) -> Option<&Arc<GpuModelResource>> {
        self.models.get(id).map(|prepared| &prepared.resource)
    }

    pub(crate) fn mesh(&self, id: &ResourceId) -> Option<&Arc<GpuMeshResource>> {
        self.meshes.get(id).map(|prepared| &prepared.resource)
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
        self.load_model_asset(*id)
            .map(|asset| asset.management_record(*id))
    }

    #[allow(dead_code)]
    pub(crate) fn model_asset_management_records(&self) -> Vec<ModelAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Model)
            .into_iter()
            .filter_map(|model_id| self.model_asset_management_record(&model_id))
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn model_asset_management_record_set(&self) -> ModelAssetManagementRecordSet {
        ModelAssetManagementRecordSet::from_records(self.model_asset_management_records())
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
        self.asset_manager
            .load_mesh_asset(*id)
            .ok()
            .map(|asset| asset.management_record(*id))
    }

    #[allow(dead_code)]
    pub(crate) fn mesh_asset_management_record_results(
        &self,
    ) -> Vec<(
        ResourceId,
        Result<MeshAssetManagementRecord, MeshValidationError>,
    )> {
        self.asset_ids_by_kind(ResourceKind::Mesh)
            .into_iter()
            .filter_map(|mesh_id| {
                self.mesh_asset_management_record(&mesh_id)
                    .map(|result| (mesh_id, result))
            })
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn mesh_asset_management_record_set(&self) -> MeshAssetManagementRecordSet {
        MeshAssetManagementRecordSet::from_results(self.mesh_asset_management_record_results())
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
        self.asset_manager
            .load_scene_asset(*id)
            .ok()
            .map(|asset| asset.management_record(*id))
    }

    #[allow(dead_code)]
    pub(crate) fn scene_asset_management_records(&self) -> Vec<SceneAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Scene)
            .into_iter()
            .filter_map(|scene_id| self.scene_asset_management_record(&scene_id))
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn scene_asset_management_record_set(&self) -> SceneAssetManagementRecordSet {
        SceneAssetManagementRecordSet::from_records(self.scene_asset_management_records())
    }

    #[allow(dead_code)]
    pub(crate) fn scene_entity_management_records(&self) -> Vec<SceneEntityManagementRecord> {
        self.scene_asset_management_records()
            .into_iter()
            .flat_map(|record| record.entity_management_records())
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn scene_entity_management_record_set(&self) -> SceneEntityManagementRecordSet {
        SceneEntityManagementRecordSet::from_records(self.scene_entity_management_records())
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
        self.asset_manager
            .load_material_asset(*id)
            .ok()
            .map(|asset| asset.management_record(*id))
    }

    #[allow(dead_code)]
    pub(crate) fn material_asset_management_records(&self) -> Vec<MaterialAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Material)
            .into_iter()
            .filter_map(|material_id| self.material_asset_management_record(&material_id))
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn material_asset_management_record_set(&self) -> MaterialAssetManagementRecordSet {
        MaterialAssetManagementRecordSet::from_records(self.material_asset_management_records())
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_readiness_report(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderReadinessReport> {
        self.asset_manager
            .load_shader_asset(*id)
            .ok()
            .map(|asset| asset.readiness_report())
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_readiness_summary(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetReadinessSummary> {
        self.shader_asset_readiness_report(id)
            .map(|report| report.summary())
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_management_record(
        &self,
        id: &ResourceId,
    ) -> Option<ShaderAssetManagementRecord> {
        self.shader_asset_readiness_report(id)
            .map(|report| report.management_record(*id))
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_management_records(&self) -> Vec<ShaderAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Shader)
            .into_iter()
            .filter_map(|shader_id| self.shader_asset_management_record(&shader_id))
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn shader_asset_management_record_set(&self) -> ShaderAssetManagementRecordSet {
        ShaderAssetManagementRecordSet::from_records(self.shader_asset_management_records())
    }

    #[allow(dead_code)]
    pub(crate) fn asset_management_record_sets(&self) -> AssetManagementRecordSets {
        AssetManagementRecordSets::from_record_sets(
            self.model_asset_management_record_set(),
            self.mesh_asset_management_record_set(),
            self.scene_asset_management_record_set(),
            self.scene_entity_management_record_set(),
            self.material_asset_management_record_set(),
            self.material_management_record_set(),
            self.shader_asset_management_record_set(),
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

    pub(crate) fn material(&self, id: &ResourceId) -> Option<&MaterialRuntime> {
        self.materials.get(id).map(|prepared| &prepared.runtime)
    }

    pub(crate) fn material_uniform(&self, id: &ResourceId) -> Arc<GpuMaterialUniformResource> {
        self.materials
            .get(id)
            .map(|prepared| prepared.uniform.clone())
            .unwrap_or_else(|| self.fallback_material_uniform.clone())
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
                            unlit: descriptor.unlit,
                            base_color_texture: self
                                .resolve_texture_reference(
                                    "base_color_texture",
                                    descriptor.base_color_texture.as_ref(),
                                )
                                .id(),
                            normal_texture: self
                                .resolve_texture_reference(
                                    "normal_texture",
                                    descriptor.normal_texture.as_ref(),
                                )
                                .id(),
                            metallic_roughness_texture: self
                                .resolve_texture_reference(
                                    "metallic_roughness_texture",
                                    descriptor.metallic_roughness_texture.as_ref(),
                                )
                                .id(),
                            occlusion_texture: self
                                .resolve_texture_reference(
                                    "occlusion_texture",
                                    descriptor.occlusion_texture.as_ref(),
                                )
                                .id(),
                            emissive_texture: self
                                .resolve_texture_reference(
                                    "emissive_texture",
                                    descriptor.emissive_texture.as_ref(),
                                )
                                .id(),
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
