//! SceneAsset 是磁盘场景与 World 的中间模型；import_scene 解析后由依赖提取器登记直接引用，scene/world/project_io 再负责运行时实例化和保存。

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
    /// Validates every opaque component reference table before dependencies or digests are
    /// published.  This is fallible so project and cache consumers can reject malformed rows
    /// before they allocate a World or mutate an index.
    pub fn validate_component_references(&self) -> Result<(), String> {
        self.entities
            .iter()
            .enumerate()
            .try_for_each(|(entity_index, entity)| {
                entity
                    .validate_direct_references()
                    .map_err(|error| format!("scene entity {entity_index}: {error}"))
            })
    }

    pub fn to_project_toml_string(
        &self,
        resolver: impl FnMut(
            &AssetReference,
        ) -> Result<
            zircon_runtime_interface::project::PersistedAssetReference,
            ReferenceResolutionError,
        >,
    ) -> Result<String, ProjectDocumentError> {
        self.validate_component_references()
            .map_err(|message| ProjectDocumentError::Schema { message })?;
        crate::asset::assets::project_document::serialize_scene(self, resolver)
    }

    pub fn from_project_toml_str(
        document: &str,
        resolver: impl FnMut(
            &zircon_runtime_interface::project::PersistedAssetReference,
        ) -> Result<AssetReference, ReferenceResolutionError>,
    ) -> Result<Self, ProjectDocumentError> {
        let scene = crate::asset::assets::project_document::deserialize_scene(document, resolver)?;
        scene
            .validate_component_references()
            .map_err(|message| ProjectDocumentError::Schema { message })?;
        Ok(scene)
    }

    #[cfg(test)]
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    #[cfg(test)]
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// 供资产注册表建立场景的直接依赖边；保留实体顺序和重复引用，不做传递闭包。
    pub fn direct_references(&self) -> Vec<AssetReference> {
        // Keep the historical infallible projection for callers that only need a best-effort
        // view. Fallible asset registration/import paths call validate_component_references()
        // before this method and therefore cannot publish malformed generic dependencies.
        let capacity = self
            .entities
            .iter()
            .map(SceneEntityAsset::direct_reference_count)
            .fold(0usize, usize::saturating_add);
        let mut references = Vec::with_capacity(capacity);
        for entity in &self.entities {
            entity.append_direct_references(&mut references);
        }
        references
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

#[cfg(test)]
#[path = "asset/tests/performance_tests.rs"]
mod performance_tests;
