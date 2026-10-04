use std::sync::Arc;

use crate::asset::{
    AssetId, AssetManagementFamilyIssueBucket, AssetManagementFamilyIssueIndex,
    AssetManagementFamilyIssueView, AssetManagementFamilyStatus, AssetManagementFamilyStatusIndex,
    AssetManagementFamilyStatusView, AssetManagementFamilySummary, AssetManagementOverview,
    AssetManagementRecordSets, ImportedAsset, MaterialAsset, MaterialAssetManagementRecord,
    MaterialAssetManagementRecordSet, MeshAsset, MeshAssetManagementRecord,
    MeshAssetManagementRecordSet, MeshValidationError, ModelAsset, ModelAssetManagementRecord,
    ModelAssetManagementRecordSet, SceneAsset, SceneAssetManagementRecord,
    SceneAssetManagementRecordSet, SceneEntityManagementRecord, SceneEntityManagementRecordSet,
    ShaderAsset, ShaderAssetManagementRecord, ShaderAssetManagementRecordSet,
    ShaderAssetReadinessSummary, ShaderReadinessReport,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;
#[cfg(feature = "profiling")]
use crate::core::resource::ResourceManagementScan;
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceData, ResourceHandle, ResourceId,
    ResourceKind, ResourceManagementGeneration, ResourceManagementQuery, ResourceMarker,
    ResourceScheme, ResourceState, SceneMarker, ShaderMarker,
};

use super::super::builtins::builtin_resource;
use super::management_generation::ProjectAssetManagementGeneration;
use super::ProjectAssetManager;

#[cfg(feature = "profiling")]
const PROFILE_STREAM: &str = "runtime";
#[cfg(feature = "profiling")]
const RESOURCE_MANAGEMENT_PROFILE_CATEGORY: &str = "resource_management";

#[cfg(feature = "profiling")]
fn resource_management_profile_scope(
    name: &'static str,
) -> Option<crate::core::runtime::diagnostics::profiling::ProfileScope> {
    use crate::core::runtime::diagnostics::profiling::{
        capture_active, ProfileFrameContext, ProfileScope,
    };

    (capture_active() && ProfileFrameContext::is_active())
        .then(|| ProfileScope::enter(PROFILE_STREAM, RESOURCE_MANAGEMENT_PROFILE_CATEGORY, name))
}

#[cfg(feature = "profiling")]
fn record_completed_resource_management_scan(scan: &ResourceManagementScan) {
    use crate::core::runtime::diagnostics::profiling::record_counter_batch;

    let metrics = scan.diagnostics();
    record_counter_batch(
        PROFILE_STREAM,
        &[
            ("resource_management.scan.instances", 1.0),
            (
                "resource_management.scan.matching_rows",
                scan.total_matching_count() as f64,
            ),
            (
                "resource_management.scan.rows_emitted",
                metrics.rows_emitted() as f64,
            ),
            (
                "resource_management.scan.shard_candidate_checks",
                metrics.shard_candidate_checks() as f64,
            ),
            (
                "resource_management.scan.filtered_rows_skipped",
                metrics.filtered_rows_skipped() as f64,
            ),
        ],
    );
}

struct AssetManagementKindIds {
    models: Vec<ResourceId>,
    meshes: Vec<ResourceId>,
    scenes: Vec<ResourceId>,
    materials: Vec<ResourceId>,
    shaders: Vec<ResourceId>,
}

impl ProjectAssetManager {
    // Publication already holds the project-generation write fence. Reading a lazy asset through
    // load_* here would try to acquire the same fence again in ensure_resident and deadlock.
    fn published_management_asset<TMarker, TAsset>(
        &self,
        id: AssetId,
        extract: impl FnOnce(ImportedAsset) -> Option<TAsset>,
    ) -> Option<TAsset>
    where
        TMarker: ResourceMarker,
        TAsset: ResourceData + Clone,
    {
        let resources = self.resource_manager();
        if let Some(asset) = resources.get::<TMarker, TAsset>(ResourceHandle::new(id)) {
            return Some(asset.as_ref().clone());
        }
        let metadata = resources.registry().get(id).cloned()?;
        if metadata.state != ResourceState::Ready {
            return None;
        }
        let imported = match metadata.primary_locator.scheme() {
            ResourceScheme::Res | ResourceScheme::Library | ResourceScheme::Package => {
                let prepared = self
                    .project_read()
                    .as_ref()?
                    .prepare_artifact_read_by_id(id)
                    .ok()?;
                prepared.read().ok()?
            }
            ResourceScheme::Builtin => {
                builtin_resource(&metadata.primary_locator, |locator, candidate| {
                    locator.to_string() == candidate
                })?
            }
            ResourceScheme::Memory => return None,
        };
        extract(imported)
    }

    fn asset_ids_by_kind(&self, kind: ResourceKind) -> Vec<ResourceId> {
        self.current_asset_management_generation()
            .ids_by_kind(kind)
            .to_vec()
    }

    fn asset_ids_for_management_record_sets(
        &self,
        generation: &Arc<ResourceManagementGeneration>,
    ) -> AssetManagementKindIds {
        #[cfg(feature = "profiling")]
        let profile_scope =
            resource_management_profile_scope("project_asset_manager.record_sets_scan");
        let summary = generation.summary();
        let mut ids = AssetManagementKindIds {
            models: Vec::with_capacity(summary.kind(ResourceKind::Model).total_count),
            meshes: Vec::with_capacity(summary.kind(ResourceKind::Mesh).total_count),
            scenes: Vec::with_capacity(summary.kind(ResourceKind::Scene).total_count),
            materials: Vec::with_capacity(summary.kind(ResourceKind::Material).total_count),
            shaders: Vec::with_capacity(summary.kind(ResourceKind::Shader).total_count),
        };
        let mut scan = generation.scan(ResourceManagementQuery {
            kind: None,
            state: None,
        });
        while let Some(row) = scan.next_row() {
            match row.kind {
                ResourceKind::Model => ids.models.push(row.id),
                ResourceKind::Mesh => ids.meshes.push(row.id),
                ResourceKind::Scene => ids.scenes.push(row.id),
                ResourceKind::Material => ids.materials.push(row.id),
                ResourceKind::Shader => ids.shaders.push(row.id),
                _ => {}
            }
        }
        #[cfg(feature = "profiling")]
        if profile_scope.is_some() {
            record_completed_resource_management_scan(&scan);
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
        self.current_asset_management_generation()
            .model_records()
            .to_vec()
    }

    fn model_asset_management_records_for_ids(
        &self,
        model_ids: Vec<ResourceId>,
    ) -> Vec<ModelAssetManagementRecord> {
        model_ids
            .into_iter()
            .filter_map(|model_id| {
                self.published_management_asset::<ModelMarker, ModelAsset>(model_id, |asset| {
                    match asset {
                        ImportedAsset::Model(model) => Some(model),
                        _ => None,
                    }
                })
                .map(|asset| asset.management_record(model_id))
            })
            .collect()
    }

    pub fn model_asset_management_record_set(&self) -> ModelAssetManagementRecordSet {
        self.current_asset_management_generation()
            .model_record_set()
            .clone()
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
        self.mesh_asset_management_record_results_for_ids(
            self.asset_ids_by_kind(ResourceKind::Mesh),
        )
    }

    fn mesh_asset_management_record_results_for_ids(
        &self,
        mesh_ids: Vec<ResourceId>,
    ) -> Vec<(
        ResourceId,
        Result<MeshAssetManagementRecord, MeshValidationError>,
    )> {
        mesh_ids
            .into_iter()
            .filter_map(|mesh_id| {
                self.mesh_asset_management_record(mesh_id)
                    .map(|result| (mesh_id, result))
            })
            .collect()
    }

    pub fn mesh_asset_management_record_set(&self) -> MeshAssetManagementRecordSet {
        self.current_asset_management_generation()
            .mesh_record_set()
            .clone()
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
        self.current_asset_management_generation()
            .scene_records()
            .to_vec()
    }

    fn scene_asset_management_records_for_ids(
        &self,
        scene_ids: Vec<ResourceId>,
    ) -> Vec<SceneAssetManagementRecord> {
        scene_ids
            .into_iter()
            .filter_map(|scene_id| {
                self.published_management_asset::<SceneMarker, SceneAsset>(scene_id, |asset| {
                    match asset {
                        ImportedAsset::Scene(scene) => Some(scene),
                        _ => None,
                    }
                })
                .map(|asset| asset.management_record(scene_id))
            })
            .collect()
    }

    pub fn scene_asset_management_record_set(&self) -> SceneAssetManagementRecordSet {
        self.current_asset_management_generation()
            .scene_record_set()
            .clone()
    }

    pub fn scene_entity_management_records(&self) -> Vec<SceneEntityManagementRecord> {
        self.current_asset_management_generation()
            .scene_entity_records()
            .to_vec()
    }

    pub fn scene_entity_management_record_set(&self) -> SceneEntityManagementRecordSet {
        self.current_asset_management_generation()
            .scene_entity_record_set()
            .clone()
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
        self.current_asset_management_generation()
            .material_records()
            .to_vec()
    }

    fn material_asset_management_records_for_ids(
        &self,
        material_ids: Vec<ResourceId>,
    ) -> Vec<MaterialAssetManagementRecord> {
        material_ids
            .into_iter()
            .filter_map(|material_id| {
                self.published_management_asset::<MaterialMarker, MaterialAsset>(
                    material_id,
                    |asset| match asset {
                        ImportedAsset::Material(material) => Some(material),
                        _ => None,
                    },
                )
                .map(|asset| asset.management_record(material_id))
            })
            .collect()
    }

    pub fn material_asset_management_record_set(&self) -> MaterialAssetManagementRecordSet {
        self.current_asset_management_generation()
            .material_record_set()
            .clone()
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
        self.current_asset_management_generation()
            .shader_records()
            .to_vec()
    }

    fn shader_asset_management_records_for_ids(
        &self,
        shader_ids: Vec<ResourceId>,
    ) -> Vec<ShaderAssetManagementRecord> {
        shader_ids
            .into_iter()
            .filter_map(|shader_id| {
                self.published_management_asset::<ShaderMarker, ShaderAsset>(shader_id, |asset| {
                    match asset {
                        ImportedAsset::Shader(shader) => Some(shader),
                        _ => None,
                    }
                })
                .map(|asset| asset.readiness_report().management_record(shader_id))
            })
            .collect()
    }

    fn published_mesh_asset_management_record_results_for_ids(
        &self,
        mesh_ids: Vec<ResourceId>,
    ) -> Vec<(
        ResourceId,
        Result<MeshAssetManagementRecord, MeshValidationError>,
    )> {
        mesh_ids
            .into_iter()
            .filter_map(|mesh_id| {
                self.published_management_asset::<MeshMarker, MeshAsset>(mesh_id, |asset| {
                    match asset {
                        ImportedAsset::Mesh(mesh) => Some(mesh),
                        _ => None,
                    }
                })
                .map(|asset| (mesh_id, asset.management_record(mesh_id)))
            })
            .collect()
    }

    pub fn shader_asset_management_record_set(&self) -> ShaderAssetManagementRecordSet {
        self.current_asset_management_generation()
            .shader_record_set()
            .clone()
    }

    fn build_asset_management_record_sets(
        &self,
        generation: &Arc<ResourceManagementGeneration>,
        materials: RenderMaterialManagementRecordSet,
    ) -> AssetManagementRecordSets {
        #[cfg(feature = "profiling")]
        let _profile_scope = resource_management_profile_scope("project_asset_manager.record_sets");
        let ids = self.asset_ids_for_management_record_sets(generation);
        let model_records = self.model_asset_management_records_for_ids(ids.models);
        let mesh_results = self.published_mesh_asset_management_record_results_for_ids(ids.meshes);
        let scene_records = self.scene_asset_management_records_for_ids(ids.scenes);
        let material_records = self.material_asset_management_records_for_ids(ids.materials);
        let shader_records = self.shader_asset_management_records_for_ids(ids.shaders);
        let scene_entities = scene_records
            .iter()
            .flat_map(SceneAssetManagementRecord::entity_management_records)
            .collect();
        AssetManagementRecordSets::from_record_sets(
            ModelAssetManagementRecordSet::from_records(model_records),
            MeshAssetManagementRecordSet::from_results(mesh_results),
            SceneAssetManagementRecordSet::from_records(scene_records),
            SceneEntityManagementRecordSet::from_records(scene_entities),
            MaterialAssetManagementRecordSet::from_records(material_records),
            materials,
            ShaderAssetManagementRecordSet::from_records(shader_records),
        )
    }

    pub(crate) fn refresh_asset_management_generation(&self) {
        let project_generation = self
            .project_read()
            .as_ref()
            .map(|project| project.catalog_input_generation().sequence());
        let Some(project_generation) = project_generation else {
            if self
                .asset_management_generation_snapshot()
                .has_project_generation()
            {
                self.install_asset_management_generation(ProjectAssetManagementGeneration::empty());
            }
            return;
        };
        let resource_generation = self.resource_manager().management_generation();
        let resource_identity = resource_generation.identity();
        if self
            .asset_management_generation_snapshot()
            .is_for_generations(project_generation, &resource_identity)
        {
            return;
        }
        let records = self.build_asset_management_record_sets(
            &resource_generation,
            RenderMaterialManagementRecordSet::default(),
        );
        self.install_asset_management_generation(
            ProjectAssetManagementGeneration::from_record_sets(
                Some(project_generation),
                Some(resource_identity),
                records.models,
                records.meshes,
                records.scenes,
                records.scene_entities,
                records.material_assets,
                records.shaders,
            ),
        );
    }

    pub fn asset_management_record_sets(&self) -> AssetManagementRecordSets {
        let generation = self.current_asset_management_generation();
        AssetManagementRecordSets::from_record_sets(
            generation.model_record_set().clone(),
            generation.mesh_record_set().clone(),
            generation.scene_record_set().clone(),
            generation.scene_entity_record_set().clone(),
            generation.material_record_set().clone(),
            RenderMaterialManagementRecordSet::default(),
            generation.shader_record_set().clone(),
        )
    }

    pub fn asset_management_overview(&self) -> AssetManagementOverview {
        self.current_asset_management_generation()
            .overview()
            .clone()
    }

    pub fn asset_management_family_summaries(&self) -> Vec<AssetManagementFamilySummary> {
        self.current_asset_management_generation()
            .overview()
            .families
            .clone()
    }

    pub fn asset_management_family_status_index(&self) -> AssetManagementFamilyStatusIndex {
        self.current_asset_management_generation()
            .overview()
            .family_status_index
            .clone()
    }

    pub fn asset_management_family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        self.current_asset_management_generation()
            .overview()
            .family_status_view(status)
    }

    pub fn asset_management_family_issue_index(&self) -> AssetManagementFamilyIssueIndex {
        self.current_asset_management_generation()
            .overview()
            .family_issue_index
            .clone()
    }

    pub fn asset_management_family_issue_view(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> AssetManagementFamilyIssueView {
        self.current_asset_management_generation()
            .overview()
            .family_issue_view(bucket)
    }
}

#[cfg(test)]
#[path = "tests/management.rs"]
mod tests;
