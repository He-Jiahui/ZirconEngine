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
use crate::core::resource::{ResourceId, ResourceKind, ResourceManagementQuery};

use super::ProjectAssetManager;

impl ProjectAssetManager {
    fn asset_ids_by_kind(&self, kind: ResourceKind) -> Vec<ResourceId> {
        let generation = self.resource_manager().management_generation();
        let mut scan = generation.scan(ResourceManagementQuery {
            kind: Some(kind),
            state: None,
        });
        let mut ids = Vec::with_capacity(generation.summary().kind(kind).total_count);
        while let Some(row) = scan.next_row() {
            ids.push(row.id);
        }
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
        let scene_records = self.scene_asset_management_records();
        let scene_entities = scene_records
            .iter()
            .flat_map(SceneAssetManagementRecord::entity_management_records)
            .collect();
        AssetManagementRecordSets::from_record_sets(
            self.model_asset_management_record_set(),
            self.mesh_asset_management_record_set(),
            SceneAssetManagementRecordSet::from_records(scene_records),
            SceneEntityManagementRecordSet::from_records(scene_entities),
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

#[cfg(test)]
mod tests {
    #[test]
    fn asset_management_kind_lookup_reads_the_published_resource_generation() {
        let source = include_str!("management.rs");
        let kind_lookup = source
            .split("fn asset_ids_by_kind")
            .nth(1)
            .and_then(|source| source.split("pub fn model_asset_management_record").next())
            .expect("read asset management kind lookup");

        assert!(kind_lookup.contains("management_generation()"));
        assert!(kind_lookup.contains("ResourceManagementQuery"));
        assert!(kind_lookup.contains("generation.scan(ResourceManagementQuery"));
        assert!(kind_lookup.contains("scan.next_row()"));
        assert!(kind_lookup.contains("ids.push(row.id)"));
        assert!(!kind_lookup.contains(".registry()"));
        assert!(!kind_lookup.contains("list_resources("));
        assert!(!kind_lookup.contains("ids.sort()"));
        assert!(!kind_lookup.contains("sort_by("));
        assert!(!kind_lookup.contains("sort_by_key("));
        assert!(!kind_lookup.contains("sort_unstable"));
    }

    #[test]
    fn asset_management_aggregate_derives_scene_entities_from_one_scene_projection() {
        let source = include_str!("management.rs");
        let aggregate = source
            .split("fn asset_management_record_sets_with_prepared_materials")
            .nth(1)
            .and_then(|source| source.split("pub fn asset_management_record_sets").next())
            .expect("read asset management aggregate implementation");

        assert_eq!(
            aggregate
                .matches("self.scene_asset_management_records()")
                .count(),
            1
        );
        assert!(aggregate.contains("SceneAssetManagementRecord::entity_management_records"));
        assert!(aggregate.contains("SceneAssetManagementRecordSet::from_records(scene_records)"));
        assert!(aggregate.contains("SceneEntityManagementRecordSet::from_records(scene_entities)"));
    }

}
