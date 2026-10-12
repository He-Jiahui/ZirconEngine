use std::sync::Arc;

use crate::asset::{
    AssetManagementOverview, AssetManagementRecordSetSummary, MaterialAssetManagementRecordSet,
    MeshAssetManagementRecordSet, ModelAssetManagementRecordSet, SceneAssetManagementRecordSet,
    SceneEntityManagementRecordSet, ShaderAssetManagementRecordSet,
};
use crate::core::resource::{ResourceId, ResourceKind, ResourceManagementGenerationIdentity};

/// Immutable asset-owned management rows published with a project generation.
///
/// Renderer-prepared material rows intentionally do not live here. Graphics consumers compose
/// those rows with this asset-only projection when they need a renderer-facing payload.
/// Cache validity uses the resource publication identity; diagnostic counters are not identities.
#[derive(Clone, Debug)]
pub struct ProjectAssetManagementGeneration {
    project_generation: Option<u64>,
    resource_generation: Option<ResourceManagementGenerationIdentity>,
    overview: AssetManagementOverview,
    models: ModelAssetManagementRecordSet,
    meshes: MeshAssetManagementRecordSet,
    scenes: SceneAssetManagementRecordSet,
    scene_entities: SceneEntityManagementRecordSet,
    material_assets: MaterialAssetManagementRecordSet,
    shaders: ShaderAssetManagementRecordSet,
    model_ids: Arc<[ResourceId]>,
    mesh_ids: Arc<[ResourceId]>,
    scene_ids: Arc<[ResourceId]>,
    material_ids: Arc<[ResourceId]>,
    shader_ids: Arc<[ResourceId]>,
}

impl ProjectAssetManagementGeneration {
    pub(in crate::asset::pipeline::manager) fn empty() -> Self {
        Self::from_record_sets(
            None,
            None,
            ModelAssetManagementRecordSet::from_records(Vec::new()),
            MeshAssetManagementRecordSet::from_results(Vec::new()),
            SceneAssetManagementRecordSet::from_records(Vec::new()),
            SceneEntityManagementRecordSet::from_records(Vec::new()),
            MaterialAssetManagementRecordSet::from_records(Vec::new()),
            ShaderAssetManagementRecordSet::from_records(Vec::new()),
        )
    }

    pub(in crate::asset::pipeline::manager) fn from_record_sets(
        project_generation: Option<u64>,
        resource_generation: Option<ResourceManagementGenerationIdentity>,
        models: ModelAssetManagementRecordSet,
        meshes: MeshAssetManagementRecordSet,
        scenes: SceneAssetManagementRecordSet,
        scene_entities: SceneEntityManagementRecordSet,
        material_assets: MaterialAssetManagementRecordSet,
        shaders: ShaderAssetManagementRecordSet,
    ) -> Self {
        let overview = AssetManagementOverview::from_summary(
            AssetManagementRecordSetSummary::from_asset_record_sets(
                &models,
                &meshes,
                &scenes,
                &scene_entities,
                &material_assets,
                &shaders,
            ),
        );
        let model_ids: Arc<[ResourceId]> = Arc::from(
            models
                .records
                .iter()
                .map(|record| record.model_id)
                .collect::<Vec<_>>(),
        );
        let mut mesh_ids_vec = meshes
            .records
            .iter()
            .map(|record| record.mesh_id)
            .chain(meshes.failures.iter().map(|failure| failure.mesh_id))
            .collect::<Vec<_>>();
        mesh_ids_vec.sort_unstable();
        let mesh_ids: Arc<[ResourceId]> = Arc::from(mesh_ids_vec);
        let scene_ids: Arc<[ResourceId]> = Arc::from(
            scenes
                .records
                .iter()
                .map(|record| record.scene_id)
                .collect::<Vec<_>>(),
        );
        let material_ids: Arc<[ResourceId]> = Arc::from(
            material_assets
                .records
                .iter()
                .map(|record| record.material_id)
                .collect::<Vec<_>>(),
        );
        let shader_ids: Arc<[ResourceId]> = Arc::from(
            shaders
                .records
                .iter()
                .map(|record| record.shader_id)
                .collect::<Vec<_>>(),
        );
        Self {
            project_generation,
            resource_generation,
            overview,
            models,
            meshes,
            scenes,
            scene_entities,
            material_assets,
            shaders,
            model_ids,
            mesh_ids,
            scene_ids,
            material_ids,
            shader_ids,
        }
    }

    pub fn resource_generation_identity(&self) -> Option<&ResourceManagementGenerationIdentity> {
        self.resource_generation.as_ref()
    }

    pub(crate) fn overview(&self) -> &AssetManagementOverview {
        &self.overview
    }

    pub(crate) fn is_for_generations(
        &self,
        project_generation: u64,
        resource_generation: &ResourceManagementGenerationIdentity,
    ) -> bool {
        self.project_generation == Some(project_generation)
            && self.resource_generation.as_ref() == Some(resource_generation)
    }

    pub(crate) fn has_project_generation(&self) -> bool {
        self.project_generation.is_some()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.models.records.is_empty()
            && self.meshes.records.is_empty()
            && self.meshes.failures.is_empty()
            && self.scenes.records.is_empty()
            && self.scene_entities.records.is_empty()
            && self.material_assets.records.is_empty()
            && self.shaders.records.is_empty()
    }

    pub(crate) fn ids_by_kind(&self, kind: ResourceKind) -> &[ResourceId] {
        match kind {
            ResourceKind::Model => &self.model_ids,
            ResourceKind::Mesh => &self.mesh_ids,
            ResourceKind::Scene => &self.scene_ids,
            ResourceKind::Material => &self.material_ids,
            ResourceKind::Shader => &self.shader_ids,
            _ => &[],
        }
    }

    pub(crate) fn model_records(&self) -> &[crate::asset::ModelAssetManagementRecord] {
        &self.models.records
    }

    pub(crate) fn model_record_set(&self) -> &ModelAssetManagementRecordSet {
        &self.models
    }

    pub(crate) fn mesh_record_set(&self) -> &MeshAssetManagementRecordSet {
        &self.meshes
    }

    pub(crate) fn scene_records(&self) -> &[crate::asset::SceneAssetManagementRecord] {
        &self.scenes.records
    }

    pub(crate) fn scene_record_set(&self) -> &SceneAssetManagementRecordSet {
        &self.scenes
    }

    pub(crate) fn scene_entity_records(&self) -> &[crate::asset::SceneEntityManagementRecord] {
        &self.scene_entities.records
    }

    pub(crate) fn scene_entity_record_set(&self) -> &SceneEntityManagementRecordSet {
        &self.scene_entities
    }

    pub(crate) fn material_records(&self) -> &[crate::asset::MaterialAssetManagementRecord] {
        &self.material_assets.records
    }

    pub(crate) fn material_record_set(&self) -> &MaterialAssetManagementRecordSet {
        &self.material_assets
    }

    pub(crate) fn shader_records(&self) -> &[crate::asset::ShaderAssetManagementRecord] {
        &self.shaders.records
    }

    pub(crate) fn shader_record_set(&self) -> &ShaderAssetManagementRecordSet {
        &self.shaders
    }
}

#[cfg(test)]
#[path = "tests/management_generation.rs"]
mod tests;
