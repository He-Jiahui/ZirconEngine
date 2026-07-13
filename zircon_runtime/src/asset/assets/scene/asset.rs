use crate::asset::assets::ProjectDocumentError;
use crate::asset::{AssetReference, ReferenceResolutionError};
use crate::core::resource::ResourceId;
use serde::{Deserialize, Serialize};

use super::entity::SceneEntityAsset;
use super::management::{
    SceneAssetManagementRecord, SceneAssetOverview, SceneEntityManagementRecord,
    SceneEntityOverview,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAsset {
    pub entities: Vec<SceneEntityAsset>,
}

impl SceneAsset {
    pub fn to_project_toml_string(
        &self,
        resolver: impl FnMut(
            &AssetReference,
        ) -> Result<
            zircon_runtime_interface::project::PersistedAssetReference,
            ReferenceResolutionError,
        >,
    ) -> Result<String, ProjectDocumentError> {
        let document = crate::asset::assets::project_document::serialize_scene(self, resolver)?;
        crate::asset::assets::project_document::validate_scene(&document)?;
        Ok(document)
    }

    pub fn from_project_toml_str(
        document: &str,
        resolver: impl FnMut(
            &zircon_runtime_interface::project::PersistedAssetReference,
        ) -> Result<AssetReference, ReferenceResolutionError>,
    ) -> Result<Self, ProjectDocumentError> {
        crate::asset::assets::project_document::validate_scene(document)?;
        crate::asset::assets::project_document::deserialize_scene(document, resolver)
    }

    #[cfg(test)]
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    #[cfg(test)]
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        self.entities
            .iter()
            .flat_map(SceneEntityAsset::direct_references)
            .collect()
    }

    pub fn entity_overviews(&self) -> Vec<SceneEntityOverview> {
        self.entities
            .iter()
            .map(SceneEntityAsset::overview)
            .collect()
    }

    pub fn overview(&self) -> SceneAssetOverview {
        let entities = self.entity_overviews();
        SceneAssetOverview {
            entity_count: entities.len(),
            active_entity_count: entities.iter().filter(|entity| entity.active).count(),
            root_entity_count: entities
                .iter()
                .filter(|entity| entity.parent.is_none())
                .count(),
            camera_count: entities.iter().filter(|entity| entity.has_camera).count(),
            mesh_instance_count: entities.iter().filter(|entity| entity.has_mesh).count(),
            direct_mesh_reference_count: entities
                .iter()
                .map(|entity| entity.direct_mesh_reference_count)
                .sum(),
            mesh_primitive_binding_count: entities
                .iter()
                .map(|entity| entity.mesh_primitive_binding_count)
                .sum(),
            morph_weight_count: entities
                .iter()
                .map(|entity| entity.morph_weight_count)
                .sum(),
            mesh_material_binding_count: entities.iter().filter(|entity| entity.has_mesh).count(),
            collider_material_binding_count: entities
                .iter()
                .filter(|entity| entity.has_collider_material)
                .count(),
            light_count: entities.iter().map(SceneEntityOverview::light_count).sum(),
            physics_component_count: entities
                .iter()
                .map(SceneEntityOverview::physics_component_count)
                .sum(),
            animation_binding_count: entities
                .iter()
                .map(SceneEntityOverview::animation_binding_count)
                .sum(),
            terrain_count: entities.iter().filter(|entity| entity.has_terrain).count(),
            tilemap_count: entities.iter().filter(|entity| entity.has_tilemap).count(),
            prefab_instance_count: entities
                .iter()
                .filter(|entity| entity.has_prefab_instance)
                .count(),
            direct_reference_count: entities
                .iter()
                .map(|entity| entity.direct_reference_count)
                .sum(),
            entities,
        }
    }

    pub fn management_record(&self, scene_id: ResourceId) -> SceneAssetManagementRecord {
        SceneAssetManagementRecord {
            scene_id,
            overview: self.overview(),
        }
    }

    pub fn entity_management_records(
        &self,
        scene_id: ResourceId,
    ) -> Vec<SceneEntityManagementRecord> {
        self.management_record(scene_id).entity_management_records()
    }
}
