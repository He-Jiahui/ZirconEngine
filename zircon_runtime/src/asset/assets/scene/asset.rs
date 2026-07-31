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
        crate::asset::assets::project_document::serialize_scene(self, resolver)
    }

    pub fn from_project_toml_str(
        document: &str,
        resolver: impl FnMut(
            &zircon_runtime_interface::project::PersistedAssetReference,
        ) -> Result<AssetReference, ReferenceResolutionError>,
    ) -> Result<Self, ProjectDocumentError> {
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
        let mut overview = SceneAssetOverview {
            entity_count: entities.len(),
            active_entity_count: 0,
            root_entity_count: 0,
            camera_count: 0,
            mesh_instance_count: 0,
            direct_mesh_reference_count: 0,
            mesh_primitive_binding_count: 0,
            morph_weight_count: 0,
            mesh_material_binding_count: 0,
            collider_material_binding_count: 0,
            light_count: 0,
            physics_component_count: 0,
            animation_binding_count: 0,
            terrain_count: 0,
            tilemap_count: 0,
            prefab_instance_count: 0,
            direct_reference_count: 0,
            entities,
        };
        for entity in &overview.entities {
            overview.active_entity_count += usize::from(entity.active);
            overview.root_entity_count += usize::from(entity.parent.is_none());
            overview.camera_count += usize::from(entity.has_camera);
            overview.mesh_instance_count += usize::from(entity.has_mesh);
            overview.direct_mesh_reference_count += entity.direct_mesh_reference_count;
            overview.mesh_primitive_binding_count += entity.mesh_primitive_binding_count;
            overview.morph_weight_count += entity.morph_weight_count;
            overview.mesh_material_binding_count += usize::from(entity.has_mesh);
            overview.collider_material_binding_count += usize::from(entity.has_collider_material);
            overview.light_count += entity.light_count();
            overview.physics_component_count += entity.physics_component_count();
            overview.animation_binding_count += entity.animation_binding_count();
            overview.terrain_count += usize::from(entity.has_terrain);
            overview.tilemap_count += usize::from(entity.has_tilemap);
            overview.prefab_instance_count += usize::from(entity.has_prefab_instance);
            overview.direct_reference_count += entity.direct_reference_count;
        }
        overview
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
        self.entity_overviews()
            .into_iter()
            .map(|entity| SceneEntityManagementRecord { scene_id, entity })
            .collect()
    }
}
