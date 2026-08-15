use std::collections::BTreeSet;

use crate::asset::AssetReference;
use crate::core::resource::ResourceId;
use serde::{Deserialize, Serialize};

use super::entity::SceneEntityAsset;
use super::mesh::SceneMeshInstanceAsset;
use super::SceneMobilityAsset;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityOverview {
    pub entity: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub active: bool,
    pub render_layer_mask: u32,
    pub mobility: SceneMobilityAsset,
    pub direct_reference_count: usize,
    pub has_camera: bool,
    pub has_mesh: bool,
    pub has_direct_mesh_reference: bool,
    pub direct_mesh_reference_count: usize,
    pub mesh_primitive_binding_count: usize,
    pub morph_weight_count: usize,
    pub has_ambient_light: bool,
    pub has_directional_light: bool,
    pub has_point_light: bool,
    pub has_rect_light: bool,
    pub has_spot_light: bool,
    pub has_post_process_settings: bool,
    pub has_post_process_volume: bool,
    pub has_rigid_body: bool,
    pub has_collider: bool,
    pub has_collider_material: bool,
    pub has_joint: bool,
    pub has_animation_skeleton: bool,
    pub has_animation_player: bool,
    pub has_animation_sequence_player: bool,
    pub has_animation_graph_player: bool,
    pub has_animation_state_machine_player: bool,
    pub has_terrain: bool,
    pub has_tilemap: bool,
    pub has_prefab_instance: bool,
}

impl SceneEntityOverview {
    pub fn light_count(&self) -> usize {
        [
            self.has_ambient_light,
            self.has_directional_light,
            self.has_point_light,
            self.has_rect_light,
            self.has_spot_light,
        ]
        .into_iter()
        .filter(|present| *present)
        .count()
    }

    pub fn physics_component_count(&self) -> usize {
        [self.has_rigid_body, self.has_collider, self.has_joint]
            .into_iter()
            .filter(|present| *present)
            .count()
    }

    pub fn animation_binding_count(&self) -> usize {
        [
            self.has_animation_skeleton,
            self.has_animation_player,
            self.has_animation_sequence_player,
            self.has_animation_graph_player,
            self.has_animation_state_machine_player,
        ]
        .into_iter()
        .filter(|present| *present)
        .count()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetOverview {
    pub entity_count: usize,
    pub active_entity_count: usize,
    pub root_entity_count: usize,
    pub camera_count: usize,
    pub mesh_instance_count: usize,
    pub direct_mesh_reference_count: usize,
    pub mesh_primitive_binding_count: usize,
    pub morph_weight_count: usize,
    pub mesh_material_binding_count: usize,
    pub collider_material_binding_count: usize,
    pub light_count: usize,
    pub physics_component_count: usize,
    pub animation_binding_count: usize,
    pub terrain_count: usize,
    pub tilemap_count: usize,
    pub prefab_instance_count: usize,
    pub direct_reference_count: usize,
    pub entities: Vec<SceneEntityOverview>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetManagementRecord {
    pub scene_id: ResourceId,
    pub overview: SceneAssetOverview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityManagementRecord {
    pub scene_id: ResourceId,
    pub entity: SceneEntityOverview,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneAssetManagementRecordSetSummary {
    pub scene_count: usize,
    pub entity_count: usize,
    pub active_entity_count: usize,
    pub root_entity_count: usize,
    pub direct_reference_count: usize,
    pub camera_count: usize,
    pub mesh_instance_count: usize,
    pub direct_mesh_reference_count: usize,
    pub mesh_primitive_binding_count: usize,
    pub morph_weight_count: usize,
    pub mesh_material_binding_count: usize,
    pub collider_material_binding_count: usize,
    pub light_count: usize,
    pub physics_component_count: usize,
    pub animation_binding_count: usize,
    pub terrain_count: usize,
    pub tilemap_count: usize,
    pub prefab_instance_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAssetManagementRecordSet {
    pub records: Vec<SceneAssetManagementRecord>,
    pub summary: SceneAssetManagementRecordSetSummary,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneEntityManagementRecordSetSummary {
    pub scene_count: usize,
    pub entity_count: usize,
    pub active_entity_count: usize,
    pub root_entity_count: usize,
    pub direct_reference_count: usize,
    pub camera_count: usize,
    pub mesh_instance_count: usize,
    pub direct_mesh_reference_count: usize,
    pub mesh_primitive_binding_count: usize,
    pub morph_weight_count: usize,
    pub mesh_material_binding_count: usize,
    pub collider_material_binding_count: usize,
    pub light_count: usize,
    pub physics_component_count: usize,
    pub animation_binding_count: usize,
    pub terrain_count: usize,
    pub tilemap_count: usize,
    pub prefab_instance_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityManagementRecordSet {
    pub records: Vec<SceneEntityManagementRecord>,
    pub summary: SceneEntityManagementRecordSetSummary,
}

impl SceneAssetManagementRecordSetSummary {
    pub fn from_records(records: &[SceneAssetManagementRecord]) -> Self {
        let mut summary = Self {
            scene_count: records.len(),
            ..Self::default()
        };
        for record in records {
            let overview = &record.overview;
            summary.entity_count += overview.entity_count;
            summary.active_entity_count += overview.active_entity_count;
            summary.root_entity_count += overview.root_entity_count;
            summary.direct_reference_count += overview.direct_reference_count;
            summary.camera_count += overview.camera_count;
            summary.mesh_instance_count += overview.mesh_instance_count;
            summary.direct_mesh_reference_count += overview.direct_mesh_reference_count;
            summary.mesh_primitive_binding_count += overview.mesh_primitive_binding_count;
            summary.morph_weight_count += overview.morph_weight_count;
            summary.mesh_material_binding_count += overview.mesh_material_binding_count;
            summary.collider_material_binding_count += overview.collider_material_binding_count;
            summary.light_count += overview.light_count;
            summary.physics_component_count += overview.physics_component_count;
            summary.animation_binding_count += overview.animation_binding_count;
            summary.terrain_count += overview.terrain_count;
            summary.tilemap_count += overview.tilemap_count;
            summary.prefab_instance_count += overview.prefab_instance_count;
        }
        summary
    }
}

impl SceneEntityManagementRecordSetSummary {
    pub fn from_records(records: &[SceneEntityManagementRecord]) -> Self {
        let mut scene_ids = BTreeSet::new();
        let mut summary = Self {
            entity_count: records.len(),
            ..Self::default()
        };
        for record in records {
            scene_ids.insert(record.scene_id);
            let entity = &record.entity;
            summary.active_entity_count += usize::from(entity.active);
            summary.root_entity_count += usize::from(entity.parent.is_none());
            summary.direct_reference_count += entity.direct_reference_count;
            summary.camera_count += usize::from(entity.has_camera);
            summary.mesh_instance_count += usize::from(entity.has_mesh);
            summary.direct_mesh_reference_count += entity.direct_mesh_reference_count;
            summary.mesh_primitive_binding_count += entity.mesh_primitive_binding_count;
            summary.morph_weight_count += entity.morph_weight_count;
            summary.mesh_material_binding_count += usize::from(entity.has_mesh);
            summary.collider_material_binding_count += usize::from(entity.has_collider_material);
            summary.light_count += entity.light_count();
            summary.physics_component_count += entity.physics_component_count();
            summary.animation_binding_count += entity.animation_binding_count();
            summary.terrain_count += usize::from(entity.has_terrain);
            summary.tilemap_count += usize::from(entity.has_tilemap);
            summary.prefab_instance_count += usize::from(entity.has_prefab_instance);
        }
        summary.scene_count = scene_ids.len();
        summary
    }
}

impl SceneAssetManagementRecordSet {
    pub fn from_records(mut records: Vec<SceneAssetManagementRecord>) -> Self {
        records.sort_by_key(|record| record.scene_id);
        let summary = SceneAssetManagementRecordSetSummary::from_records(&records);
        Self { records, summary }
    }
}

impl SceneEntityManagementRecordSet {
    pub fn from_records(mut records: Vec<SceneEntityManagementRecord>) -> Self {
        records.sort_by_key(|record| (record.scene_id, record.entity.entity));
        let summary = SceneEntityManagementRecordSetSummary::from_records(&records);
        Self { records, summary }
    }
}

impl SceneAssetManagementRecord {
    pub fn entity_management_records(&self) -> Vec<SceneEntityManagementRecord> {
        self.overview
            .entities
            .iter()
            .cloned()
            .map(|entity| SceneEntityManagementRecord {
                scene_id: self.scene_id,
                entity,
            })
            .collect()
    }
}

impl SceneEntityAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::new();
        if let Some(camera) = &self.camera {
            references.extend(camera.direct_references());
        }
        if let Some(mesh) = &self.mesh {
            references.extend(mesh.direct_references());
        }
        if let Some(collider) = &self.collider {
            references.extend(collider.material.iter().cloned());
        }
        if let Some(skeleton) = &self.animation_skeleton {
            references.push(skeleton.skeleton.clone());
        }
        if let Some(player) = &self.animation_player {
            references.push(player.clip.clone());
        }
        if let Some(player) = &self.animation_sequence_player {
            references.push(player.sequence.clone());
        }
        if let Some(player) = &self.animation_graph_player {
            references.push(player.graph.clone());
        }
        if let Some(player) = &self.animation_state_machine_player {
            references.push(player.state_machine.clone());
        }
        if let Some(terrain) = &self.terrain {
            references.push(terrain.terrain.clone());
        }
        if let Some(tilemap) = &self.tilemap {
            references.push(tilemap.tilemap.clone());
        }
        if let Some(prefab) = &self.prefab_instance {
            references.extend(prefab.direct_references());
        }
        references
    }

    pub fn direct_reference_count(&self) -> usize {
        self.camera
            .as_ref()
            .map_or(0, |camera| camera.direct_reference_count())
            + self
                .mesh
                .as_ref()
                .map_or(0, SceneMeshInstanceAsset::direct_reference_count)
            + usize::from(
                self.collider
                    .as_ref()
                    .and_then(|collider| collider.material.as_ref())
                    .is_some(),
            )
            + usize::from(self.animation_skeleton.is_some())
            + usize::from(self.animation_player.is_some())
            + usize::from(self.animation_sequence_player.is_some())
            + usize::from(self.animation_graph_player.is_some())
            + usize::from(self.animation_state_machine_player.is_some())
            + usize::from(self.terrain.is_some())
            + usize::from(self.tilemap.is_some())
            + self
                .prefab_instance
                .as_ref()
                .map_or(0, |prefab| prefab.direct_reference_count())
    }

    pub fn overview(&self) -> SceneEntityOverview {
        let direct_mesh_reference_count = self
            .mesh
            .as_ref()
            .map(SceneMeshInstanceAsset::direct_mesh_reference_count)
            .unwrap_or(0);
        let mesh_primitive_binding_count = self
            .mesh
            .as_ref()
            .map(SceneMeshInstanceAsset::primitive_binding_count)
            .unwrap_or(0);
        let morph_weight_count = self
            .mesh
            .as_ref()
            .map(SceneMeshInstanceAsset::morph_weight_count)
            .unwrap_or(0);
        SceneEntityOverview {
            entity: self.entity,
            name: self.name.clone(),
            parent: self.parent,
            active: self.active,
            render_layer_mask: self.render_layer_mask,
            mobility: self.mobility,
            direct_reference_count: self.direct_reference_count(),
            has_camera: self.camera.is_some(),
            has_mesh: self.mesh.is_some(),
            has_direct_mesh_reference: direct_mesh_reference_count > 0,
            direct_mesh_reference_count,
            mesh_primitive_binding_count,
            morph_weight_count,
            has_ambient_light: self.ambient_light.is_some(),
            has_directional_light: self.directional_light.is_some(),
            has_point_light: self.point_light.is_some(),
            has_rect_light: self.rect_light.is_some(),
            has_spot_light: self.spot_light.is_some(),
            has_post_process_settings: self
                .camera
                .as_ref()
                .and_then(|camera| camera.post_process_settings.as_ref())
                .is_some(),
            has_post_process_volume: self.post_process_volume.is_some(),
            has_rigid_body: self.rigid_body.is_some(),
            has_collider: self.collider.is_some(),
            has_collider_material: self
                .collider
                .as_ref()
                .and_then(|collider| collider.material.as_ref())
                .is_some(),
            has_joint: self.joint.is_some(),
            has_animation_skeleton: self.animation_skeleton.is_some(),
            has_animation_player: self.animation_player.is_some(),
            has_animation_sequence_player: self.animation_sequence_player.is_some(),
            has_animation_graph_player: self.animation_graph_player.is_some(),
            has_animation_state_machine_player: self.animation_state_machine_player.is_some(),
            has_terrain: self.terrain.is_some(),
            has_tilemap: self.tilemap.is_some(),
            has_prefab_instance: self.prefab_instance.is_some(),
        }
    }
}
