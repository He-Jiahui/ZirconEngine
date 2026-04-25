use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::PhysicsMaterialMetadata;
use crate::core::math::Real;
use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;

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
    pub fov_y_radians: Real,
    pub z_near: Real,
    pub z_far: Real,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMeshInstanceAsset {
    pub model: AssetReference,
    pub material: AssetReference,
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
    pub directional_light: Option<SceneDirectionalLightAsset>,
    #[serde(default)]
    pub point_light: Option<ScenePointLightAsset>,
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
}

const fn default_scene_active() -> bool {
    true
}

const fn default_render_layer_mask() -> u32 {
    0x0000_0001
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
