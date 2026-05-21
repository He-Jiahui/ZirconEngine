use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::PhysicsMaterialMetadata;
use crate::core::framework::render::{
    ProjectionMode, RenderCameraClearColor, DEFAULT_CAMERA_EXPOSURE_EV100,
    DEFAULT_CAMERA_MSAA_SAMPLES,
};
use crate::core::math::Real;
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
pub struct SceneMeshInstanceAsset {
    pub model: AssetReference,
    pub material: AssetReference,
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
        let mut references = Vec::new();
        for entity in &self.entities {
            if let Some(camera) = &entity.camera {
                references.extend(camera.direct_references());
            }
            if let Some(mesh) = &entity.mesh {
                references.push(mesh.model.clone());
                references.push(mesh.material.clone());
            }
            if let Some(collider) = &entity.collider {
                references.extend(collider.material.iter().cloned());
            }
            if let Some(skeleton) = &entity.animation_skeleton {
                references.push(skeleton.skeleton.clone());
            }
            if let Some(player) = &entity.animation_player {
                references.push(player.clip.clone());
            }
            if let Some(player) = &entity.animation_sequence_player {
                references.push(player.sequence.clone());
            }
            if let Some(player) = &entity.animation_graph_player {
                references.push(player.graph.clone());
            }
            if let Some(player) = &entity.animation_state_machine_player {
                references.push(player.state_machine.clone());
            }
            if let Some(terrain) = &entity.terrain {
                references.push(terrain.terrain.clone());
            }
            if let Some(tilemap) = &entity.tilemap {
                references.push(tilemap.tilemap.clone());
            }
            if let Some(prefab) = &entity.prefab_instance {
                references.extend(prefab.direct_references());
            }
        }
        references
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
