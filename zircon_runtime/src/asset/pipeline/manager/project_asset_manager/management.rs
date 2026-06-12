use crate::asset::{
    AssetManagementFamilyIssueBucket, AssetManagementFamilyIssueIndex,
    AssetManagementFamilyIssueView, AssetManagementFamilyStatus, AssetManagementFamilyStatusIndex,
    AssetManagementFamilyStatusView, AssetManagementFamilySummary, AssetManagementOverview,
    AssetManagementRecordSets, MaterialAssetManagementRecord, MaterialAssetManagementRecordSet,
    MeshAssetManagementRecord, MeshAssetManagementRecordSet, MeshValidationError,
    ModelAssetManagementRecord, ModelAssetManagementRecordSet, SceneAssetManagementRecord,
    SceneAssetManagementRecordSet, SceneEntityManagementRecord, SceneEntityManagementRecordSet,
    ShaderAssetManagementRecord, ShaderAssetManagementRecordSet, ShaderAssetReadinessSummary,
    ShaderReadinessReport,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;
use crate::core::resource::{ResourceId, ResourceKind};

use super::ProjectAssetManager;

impl ProjectAssetManager {
    fn asset_ids_by_kind(&self, kind: ResourceKind) -> Vec<ResourceId> {
        let mut ids = {
            let resource_manager = self.resource_manager();
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

    pub fn model_asset_management_record(
        &self,
        id: ResourceId,
    ) -> Option<ModelAssetManagementRecord> {
        self.load_model_asset(id)
            .ok()
            .map(|asset| asset.management_record(id))
    }

    pub fn model_asset_management_records(&self) -> Vec<ModelAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Model)
            .into_iter()
            .filter_map(|model_id| self.model_asset_management_record(model_id))
            .collect()
    }

    pub fn model_asset_management_record_set(&self) -> ModelAssetManagementRecordSet {
        ModelAssetManagementRecordSet::from_records(self.model_asset_management_records())
    }

    pub fn mesh_asset_management_record(
        &self,
        id: ResourceId,
    ) -> Option<Result<MeshAssetManagementRecord, MeshValidationError>> {
        self.load_mesh_asset(id)
            .ok()
            .map(|asset| asset.management_record(id))
    }

    pub fn mesh_asset_management_record_results(
        &self,
    ) -> Vec<(
        ResourceId,
        Result<MeshAssetManagementRecord, MeshValidationError>,
    )> {
        self.asset_ids_by_kind(ResourceKind::Mesh)
            .into_iter()
            .filter_map(|mesh_id| {
                self.mesh_asset_management_record(mesh_id)
                    .map(|result| (mesh_id, result))
            })
            .collect()
    }

    pub fn mesh_asset_management_record_set(&self) -> MeshAssetManagementRecordSet {
        MeshAssetManagementRecordSet::from_results(self.mesh_asset_management_record_results())
    }

    pub fn scene_asset_management_record(
        &self,
        id: ResourceId,
    ) -> Option<SceneAssetManagementRecord> {
        self.load_scene_asset(id)
            .ok()
            .map(|asset| asset.management_record(id))
    }

    pub fn scene_asset_management_records(&self) -> Vec<SceneAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Scene)
            .into_iter()
            .filter_map(|scene_id| self.scene_asset_management_record(scene_id))
            .collect()
    }

    pub fn scene_asset_management_record_set(&self) -> SceneAssetManagementRecordSet {
        SceneAssetManagementRecordSet::from_records(self.scene_asset_management_records())
    }

    pub fn scene_entity_management_records(&self) -> Vec<SceneEntityManagementRecord> {
        self.scene_asset_management_records()
            .into_iter()
            .flat_map(|record| record.entity_management_records())
            .collect()
    }

    pub fn scene_entity_management_record_set(&self) -> SceneEntityManagementRecordSet {
        SceneEntityManagementRecordSet::from_records(self.scene_entity_management_records())
    }

    pub fn material_asset_management_record(
        &self,
        id: ResourceId,
    ) -> Option<MaterialAssetManagementRecord> {
        self.load_material_asset(id)
            .ok()
            .map(|asset| asset.management_record(id))
    }

    pub fn material_asset_management_records(&self) -> Vec<MaterialAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Material)
            .into_iter()
            .filter_map(|material_id| self.material_asset_management_record(material_id))
            .collect()
    }

    pub fn material_asset_management_record_set(&self) -> MaterialAssetManagementRecordSet {
        MaterialAssetManagementRecordSet::from_records(self.material_asset_management_records())
    }

    pub fn shader_asset_readiness_report(&self, id: ResourceId) -> Option<ShaderReadinessReport> {
        self.load_shader_asset(id)
            .ok()
            .map(|asset| asset.readiness_report())
    }

    pub fn shader_asset_readiness_summary(
        &self,
        id: ResourceId,
    ) -> Option<ShaderAssetReadinessSummary> {
        self.shader_asset_readiness_report(id)
            .map(|report| report.summary())
    }

    pub fn shader_asset_management_record(
        &self,
        id: ResourceId,
    ) -> Option<ShaderAssetManagementRecord> {
        self.shader_asset_readiness_report(id)
            .map(|report| report.management_record(id))
    }

    pub fn shader_asset_management_records(&self) -> Vec<ShaderAssetManagementRecord> {
        self.asset_ids_by_kind(ResourceKind::Shader)
            .into_iter()
            .filter_map(|shader_id| self.shader_asset_management_record(shader_id))
            .collect()
    }

    pub fn shader_asset_management_record_set(&self) -> ShaderAssetManagementRecordSet {
        ShaderAssetManagementRecordSet::from_records(self.shader_asset_management_records())
    }

    pub(crate) fn asset_management_record_sets_with_prepared_materials(
        &self,
        materials: RenderMaterialManagementRecordSet,
    ) -> AssetManagementRecordSets {
        AssetManagementRecordSets::from_record_sets(
            self.model_asset_management_record_set(),
            self.mesh_asset_management_record_set(),
            self.scene_asset_management_record_set(),
            self.scene_entity_management_record_set(),
            self.material_asset_management_record_set(),
            materials,
            self.shader_asset_management_record_set(),
        )
    }

    pub fn asset_management_record_sets(&self) -> AssetManagementRecordSets {
        self.asset_management_record_sets_with_prepared_materials(
            RenderMaterialManagementRecordSet::default(),
        )
    }

    pub fn asset_management_overview(&self) -> AssetManagementOverview {
        self.asset_management_record_sets().overview()
    }

    pub fn asset_management_family_summaries(&self) -> Vec<AssetManagementFamilySummary> {
        self.asset_management_record_sets().families
    }

    pub fn asset_management_family_status_index(&self) -> AssetManagementFamilyStatusIndex {
        self.asset_management_record_sets().family_status_index
    }

    pub fn asset_management_family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        self.asset_management_record_sets()
            .family_status_view(status)
    }

    pub fn asset_management_family_issue_index(&self) -> AssetManagementFamilyIssueIndex {
        self.asset_management_record_sets().family_issue_index
    }

    pub fn asset_management_family_issue_view(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> AssetManagementFamilyIssueView {
        self.asset_management_record_sets()
            .family_issue_view(bucket)
    }
}
