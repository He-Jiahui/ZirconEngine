use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::PhysicsMaterialMetadata;
use crate::core::framework::render::{
    ProjectionMode, RenderCameraClearColor, DEFAULT_CAMERA_EXPOSURE_EV100,
    DEFAULT_CAMERA_MSAA_SAMPLES,
};
use crate::core::math::Real;
use crate::core::resource::ResourceId;
use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;

use super::PrefabInstanceAsset;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformAsset {
    pub translation: [Real; 3],
    pub rotation: [Real; 4],
    pub scale: [Real; 3],
}

impl Default for TransformAsset {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneCameraAsset {
    #[serde(default)]
    pub projection_mode: ProjectionMode,
    #[serde(default = "default_camera_fov_y_radians")]
    pub fov_y_radians: Real,
    #[serde(default = "default_camera_ortho_size")]
    pub ortho_size: Real,
    #[serde(default = "default_camera_z_near")]
    pub z_near: Real,
    #[serde(default = "default_camera_z_far")]
    pub z_far: Real,
    #[serde(default)]
    pub target: SceneCameraTargetAsset,
    #[serde(default)]
    pub viewport: Option<SceneViewportRectAsset>,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default = "default_camera_exposure_ev100")]
    pub exposure_ev100: Real,
    #[serde(default)]
    pub clear_color: RenderCameraClearColor,
    #[serde(default = "default_camera_msaa_samples")]
    pub msaa_samples: u32,
}

impl Default for SceneCameraAsset {
    fn default() -> Self {
        Self {
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: default_camera_fov_y_radians(),
            ortho_size: default_camera_ortho_size(),
            z_near: default_camera_z_near(),
            z_far: default_camera_z_far(),
            target: SceneCameraTargetAsset::default(),
            viewport: None,
            order: 0,
            active: true,
            hdr: false,
            exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
            clear_color: RenderCameraClearColor::default(),
            msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
        }
    }
}

impl SceneCameraAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        match &self.target {
            SceneCameraTargetAsset::Texture { texture } => vec![texture.clone()],
            SceneCameraTargetAsset::PrimarySurface | SceneCameraTargetAsset::Headless { .. } => {
                Vec::new()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneCameraTargetAsset {
    PrimarySurface,
    Texture { texture: AssetReference },
    Headless { size: [u32; 2] },
}

impl Default for SceneCameraTargetAsset {
    fn default() -> Self {
        Self::PrimarySurface
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneViewportRectAsset {
    pub physical_position: [u32; 2],
    pub physical_size: [u32; 2],
    #[serde(default)]
    pub depth_min: Real,
    #[serde(default = "default_viewport_depth_max")]
    pub depth_max: Real,
}

impl Default for SceneViewportRectAsset {
    fn default() -> Self {
        Self {
            physical_position: [0, 0],
            physical_size: [1, 1],
            depth_min: 0.0,
            depth_max: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMeshPrimitiveBindingAsset {
    pub mesh: AssetReference,
    pub material: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMeshInstanceAsset {
    pub model: AssetReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<AssetReference>,
    pub material: AssetReference,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primitives: Vec<SceneMeshPrimitiveBindingAsset>,
}

impl SceneMeshInstanceAsset {
    pub fn direct_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::with_capacity(3 + (self.primitives.len() * 2));
        references.push(self.model.clone());
        references.extend(self.mesh.iter().cloned());
        references.push(self.material.clone());
        for primitive in &self.primitives {
            references.push(primitive.mesh.clone());
            references.push(primitive.material.clone());
        }
        references
    }

    pub fn direct_mesh_reference_count(&self) -> usize {
        usize::from(self.mesh.is_some()) + self.primitives.len()
    }

    pub fn primitive_binding_count(&self) -> usize {
        self.primitives.len()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAmbientLightAsset {
    #[serde(default = "default_light_color")]
    pub color: [Real; 3],
    #[serde(default = "default_ambient_light_intensity")]
    pub intensity: Real,
    #[serde(default = "default_true")]
    pub affects_lightmapped_meshes: bool,
}

impl Default for SceneAmbientLightAsset {
    fn default() -> Self {
        Self {
            color: default_light_color(),
            intensity: default_ambient_light_intensity(),
            affects_lightmapped_meshes: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDirectionalLightAsset {
    pub direction: [Real; 3],
    pub color: [Real; 3],
    pub intensity: Real,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenePointLightAsset {
    pub color: [Real; 3],
    pub intensity: Real,
    pub range: Real,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneRectLightAsset {
    #[serde(default = "default_light_color")]
    pub color: [Real; 3],
    #[serde(default = "default_rect_light_intensity")]
    pub intensity: Real,
    #[serde(default = "default_rect_light_range")]
    pub range: Real,
    #[serde(default = "default_rect_light_size")]
    pub size: [Real; 2],
}

impl Default for SceneRectLightAsset {
    fn default() -> Self {
        Self {
            color: default_light_color(),
            intensity: default_rect_light_intensity(),
            range: default_rect_light_range(),
            size: default_rect_light_size(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneSpotLightAsset {
    pub direction: [Real; 3],
    pub color: [Real; 3],
    pub intensity: Real,
    pub range: Real,
    pub inner_angle_radians: Real,
    pub outer_angle_radians: Real,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SceneMobilityAsset {
    #[default]
    Dynamic,
    Static,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneRigidBodyTypeAsset {
    Static,
    #[default]
    Dynamic,
    Kinematic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneRigidBodyAsset {
    #[serde(default)]
    pub body_type: SceneRigidBodyTypeAsset,
    #[serde(default = "default_rigid_body_mass")]
    pub mass: Real,
    #[serde(default)]
    pub linear_velocity: [Real; 3],
    #[serde(default)]
    pub angular_velocity: [Real; 3],
    #[serde(default)]
    pub linear_damping: Real,
    #[serde(default)]
    pub angular_damping: Real,
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: Real,
    #[serde(default = "default_true")]
    pub can_sleep: bool,
    #[serde(default)]
    pub lock_translation: [bool; 3],
    #[serde(default)]
    pub lock_rotation: [bool; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneColliderShapeAsset {
    Box { half_extents: [Real; 3] },
    Sphere { radius: Real },
    Capsule { radius: Real, half_height: Real },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneColliderAsset {
    pub shape: SceneColliderShapeAsset,
    #[serde(default)]
    pub sensor: bool,
    #[serde(default)]
    pub layer: u32,
    #[serde(default)]
    pub collision_group: u32,
    #[serde(default = "default_collision_mask")]
    pub collision_mask: u32,
    #[serde(default)]
    pub material: Option<AssetReference>,
    #[serde(default)]
    pub material_override: Option<PhysicsMaterialMetadata>,
    #[serde(default)]
    pub local_transform: TransformAsset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneJointKindAsset {
    #[default]
    Fixed,
    Distance,
    Hinge,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneJointAsset {
    #[serde(default)]
    pub joint_type: SceneJointKindAsset,
    #[serde(default)]
    pub connected_entity: Option<u64>,
    #[serde(default = "default_vec3_zero")]
    pub anchor: [Real; 3],
    #[serde(default = "default_vec3_up")]
    pub axis: [Real; 3],
    #[serde(default)]
    pub limits: Option<[Real; 2]>,
    #[serde(default)]
    pub collide_connected: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationSkeletonAsset {
    pub skeleton: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationPlayerAsset {
    pub clip: AssetReference,
    #[serde(default = "default_playback_speed")]
    pub playback_speed: Real,
    #[serde(default)]
    pub time_seconds: Real,
    #[serde(default = "default_animation_weight")]
    pub weight: Real,
    #[serde(default = "default_true")]
    pub looping: bool,
    #[serde(default = "default_true")]
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationSequencePlayerAsset {
    pub sequence: AssetReference,
    #[serde(default = "default_playback_speed")]
    pub playback_speed: Real,
    #[serde(default)]
    pub time_seconds: Real,
    #[serde(default = "default_true")]
    pub looping: bool,
    #[serde(default = "default_true")]
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationGraphPlayerAsset {
    pub graph: AssetReference,
    #[serde(default)]
    pub parameters: BTreeMap<String, AnimationParameterValue>,
    #[serde(default = "default_true")]
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationStateMachinePlayerAsset {
    pub state_machine: AssetReference,
    #[serde(default)]
    pub parameters: BTreeMap<String, AnimationParameterValue>,
    #[serde(default)]
    pub active_state: Option<String>,
    #[serde(default = "default_true")]
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneTerrainAsset {
    pub terrain: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneTileMapAsset {
    pub tilemap: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityAsset {
    pub entity: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub transform: TransformAsset,
    #[serde(default = "default_scene_active")]
    pub active: bool,
    #[serde(default = "default_render_layer_mask")]
    pub render_layer_mask: u32,
    #[serde(default)]
    pub mobility: SceneMobilityAsset,
    pub camera: Option<SceneCameraAsset>,
    pub mesh: Option<SceneMeshInstanceAsset>,
    #[serde(default)]
    pub ambient_light: Option<SceneAmbientLightAsset>,
    pub directional_light: Option<SceneDirectionalLightAsset>,
    #[serde(default)]
    pub point_light: Option<ScenePointLightAsset>,
    #[serde(default)]
    pub rect_light: Option<SceneRectLightAsset>,
    #[serde(default)]
    pub spot_light: Option<SceneSpotLightAsset>,
    #[serde(default)]
    pub rigid_body: Option<SceneRigidBodyAsset>,
    #[serde(default)]
    pub collider: Option<SceneColliderAsset>,
    #[serde(default)]
    pub joint: Option<SceneJointAsset>,
    #[serde(default)]
    pub animation_skeleton: Option<SceneAnimationSkeletonAsset>,
    #[serde(default)]
    pub animation_player: Option<SceneAnimationPlayerAsset>,
    #[serde(default)]
    pub animation_sequence_player: Option<SceneAnimationSequencePlayerAsset>,
    #[serde(default)]
    pub animation_graph_player: Option<SceneAnimationGraphPlayerAsset>,
    #[serde(default)]
    pub animation_state_machine_player: Option<SceneAnimationStateMachinePlayerAsset>,
    #[serde(default)]
    pub terrain: Option<SceneTerrainAsset>,
    #[serde(default)]
    pub tilemap: Option<SceneTileMapAsset>,
    #[serde(default)]
    pub prefab_instance: Option<PrefabInstanceAsset>,
}

// Read-only management DTOs keep scene authoring payloads stable while asset
// panels can inspect entity composition without walking every component type.
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
    pub has_ambient_light: bool,
    pub has_directional_light: bool,
    pub has_point_light: bool,
    pub has_rect_light: bool,
    pub has_spot_light: bool,
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
        Self {
            scene_count: records.len(),
            entity_count: records
                .iter()
                .map(|record| record.overview.entity_count)
                .sum(),
            active_entity_count: records
                .iter()
                .map(|record| record.overview.active_entity_count)
                .sum(),
            root_entity_count: records
                .iter()
                .map(|record| record.overview.root_entity_count)
                .sum(),
            direct_reference_count: records
                .iter()
                .map(|record| record.overview.direct_reference_count)
                .sum(),
            camera_count: records
                .iter()
                .map(|record| record.overview.camera_count)
                .sum(),
            mesh_instance_count: records
                .iter()
                .map(|record| record.overview.mesh_instance_count)
                .sum(),
            direct_mesh_reference_count: records
                .iter()
                .map(|record| record.overview.direct_mesh_reference_count)
                .sum(),
            mesh_primitive_binding_count: records
                .iter()
                .map(|record| record.overview.mesh_primitive_binding_count)
                .sum(),
            mesh_material_binding_count: records
                .iter()
                .map(|record| record.overview.mesh_material_binding_count)
                .sum(),
            collider_material_binding_count: records
                .iter()
                .map(|record| record.overview.collider_material_binding_count)
                .sum(),
            light_count: records
                .iter()
                .map(|record| record.overview.light_count)
                .sum(),
            physics_component_count: records
                .iter()
                .map(|record| record.overview.physics_component_count)
                .sum(),
            animation_binding_count: records
                .iter()
                .map(|record| record.overview.animation_binding_count)
                .sum(),
            terrain_count: records
                .iter()
                .map(|record| record.overview.terrain_count)
                .sum(),
            tilemap_count: records
                .iter()
                .map(|record| record.overview.tilemap_count)
                .sum(),
            prefab_instance_count: records
                .iter()
                .map(|record| record.overview.prefab_instance_count)
                .sum(),
        }
    }
}

impl SceneEntityManagementRecordSetSummary {
    pub fn from_records(records: &[SceneEntityManagementRecord]) -> Self {
        let mut scene_ids = records
            .iter()
            .map(|record| record.scene_id)
            .collect::<Vec<_>>();
        scene_ids.sort();
        scene_ids.dedup();
        Self {
            scene_count: scene_ids.len(),
            entity_count: records.len(),
            active_entity_count: records.iter().filter(|record| record.entity.active).count(),
            root_entity_count: records
                .iter()
                .filter(|record| record.entity.parent.is_none())
                .count(),
            direct_reference_count: records
                .iter()
                .map(|record| record.entity.direct_reference_count)
                .sum(),
            camera_count: records
                .iter()
                .filter(|record| record.entity.has_camera)
                .count(),
            mesh_instance_count: records
                .iter()
                .filter(|record| record.entity.has_mesh)
                .count(),
            direct_mesh_reference_count: records
                .iter()
                .map(|record| record.entity.direct_mesh_reference_count)
                .sum(),
            mesh_primitive_binding_count: records
                .iter()
                .map(|record| record.entity.mesh_primitive_binding_count)
                .sum(),
            mesh_material_binding_count: records
                .iter()
                .filter(|record| record.entity.has_mesh)
                .count(),
            collider_material_binding_count: records
                .iter()
                .filter(|record| record.entity.has_collider_material)
                .count(),
            light_count: records
                .iter()
                .map(|record| record.entity.light_count())
                .sum(),
            physics_component_count: records
                .iter()
                .map(|record| record.entity.physics_component_count())
                .sum(),
            animation_binding_count: records
                .iter()
                .map(|record| record.entity.animation_binding_count())
                .sum(),
            terrain_count: records
                .iter()
                .filter(|record| record.entity.has_terrain)
                .count(),
            tilemap_count: records
                .iter()
                .filter(|record| record.entity.has_tilemap)
                .count(),
            prefab_instance_count: records
                .iter()
                .filter(|record| record.entity.has_prefab_instance)
                .count(),
        }
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

    pub fn overview(&self) -> SceneEntityOverview {
        let direct_references = self.direct_references();
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
        SceneEntityOverview {
            entity: self.entity,
            name: self.name.clone(),
            parent: self.parent,
            active: self.active,
            render_layer_mask: self.render_layer_mask,
            mobility: self.mobility,
            direct_reference_count: direct_references.len(),
            has_camera: self.camera.is_some(),
            has_mesh: self.mesh.is_some(),
            has_direct_mesh_reference: direct_mesh_reference_count > 0,
            direct_mesh_reference_count,
            mesh_primitive_binding_count,
            has_ambient_light: self.ambient_light.is_some(),
            has_directional_light: self.directional_light.is_some(),
            has_point_light: self.point_light.is_some(),
            has_rect_light: self.rect_light.is_some(),
            has_spot_light: self.spot_light.is_some(),
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAsset {
    pub entities: Vec<SceneEntityAsset>,
}

impl SceneAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

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

const fn default_scene_active() -> bool {
    true
}

const fn default_render_layer_mask() -> u32 {
    0x0000_0001
}

fn default_camera_fov_y_radians() -> Real {
    60.0_f32.to_radians()
}

const fn default_camera_ortho_size() -> Real {
    5.0
}

const fn default_camera_z_near() -> Real {
    0.1
}

const fn default_camera_z_far() -> Real {
    200.0
}

const fn default_camera_exposure_ev100() -> Real {
    DEFAULT_CAMERA_EXPOSURE_EV100
}

const fn default_camera_msaa_samples() -> u32 {
    DEFAULT_CAMERA_MSAA_SAMPLES
}

const fn default_viewport_depth_max() -> Real {
    1.0
}

const fn default_collision_mask() -> u32 {
    u32::MAX
}

const fn default_rigid_body_mass() -> Real {
    1.0
}

const fn default_gravity_scale() -> Real {
    1.0
}

const fn default_playback_speed() -> Real {
    1.0
}

const fn default_animation_weight() -> Real {
    1.0
}

const fn default_true() -> bool {
    true
}

const fn default_vec3_zero() -> [Real; 3] {
    [0.0, 0.0, 0.0]
}

const fn default_vec3_up() -> [Real; 3] {
    [0.0, 1.0, 0.0]
}

const fn default_light_color() -> [Real; 3] {
    [1.0, 1.0, 1.0]
}

const fn default_ambient_light_intensity() -> Real {
    80.0
}

const fn default_rect_light_intensity() -> Real {
    1_000_000.0
}

const fn default_rect_light_range() -> Real {
    20.0
}

const fn default_rect_light_size() -> [Real; 2] {
    [1.0, 1.0]
}
