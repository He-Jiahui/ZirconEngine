use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::PhysicsMaterialMetadata;
use crate::core::framework::scene::Mobility;
use crate::core::math::{Mat4, Real, Transform, Vec3, Vec4};
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, MaterialMarker, ModelMarker, PhysicsMaterialMarker,
    ResourceHandle, ResourceId,
};
use serde::{Deserialize, Serialize};

use crate::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Camera,
    Cube,
    Mesh,
    DirectionalLight,
    PointLight,
    SpotLight,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Hierarchy {
    pub parent: Option<EntityId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalTransform {
    pub transform: Transform,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldMatrix(pub Mat4);

impl Default for WorldMatrix {
    fn default() -> Self {
        Self(Mat4::IDENTITY)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldTransform {
    pub transform: Transform,
}

impl Default for WorldTransform {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSelf(pub bool);

impl Default for ActiveSelf {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveInHierarchy(pub bool);

impl Default for ActiveInHierarchy {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderLayerMask(pub u32);

impl Default for RenderLayerMask {
    fn default() -> Self {
        Self(default_render_layer_mask())
    }
}

pub type Active = ActiveSelf;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov_y_radians: Real,
    pub z_near: Real,
    pub z_far: Real,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov_y_radians: 60.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 200.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshRenderer {
    pub model: ResourceHandle<ModelMarker>,
    pub material: ResourceHandle<MaterialMarker>,
    pub tint: Vec4,
}

impl MeshRenderer {
    pub fn from_handles(
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self {
            model,
            material,
            tint: Vec4::ONE,
        }
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self::from_handles(
            ResourceHandle::new(ResourceId::from_stable_label("builtin://cube")),
            ResourceHandle::new(ResourceId::from_stable_label("builtin://material/default")),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidBodyType {
    Static,
    #[default]
    Dynamic,
    Kinematic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RigidBodyComponent {
    pub body_type: RigidBodyType,
    pub mass: Real,
    pub linear_damping: Real,
    pub angular_damping: Real,
    pub gravity_scale: Real,
    pub can_sleep: bool,
    pub lock_translation: [bool; 3],
    pub lock_rotation: [bool; 3],
}

impl Default for RigidBodyComponent {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            can_sleep: true,
            lock_translation: [false; 3],
            lock_rotation: [false; 3],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ColliderShape {
    Box { half_extents: Vec3 },
    Sphere { radius: Real },
    Capsule { radius: Real, half_height: Real },
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Box {
            half_extents: Vec3::splat(0.5),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColliderComponent {
    pub shape: ColliderShape,
    pub sensor: bool,
    pub layer: u32,
    pub collision_group: u32,
    pub collision_mask: u32,
    pub material: Option<ResourceHandle<PhysicsMaterialMarker>>,
    pub material_override: Option<PhysicsMaterialMetadata>,
    pub local_transform: Transform,
}

impl Default for ColliderComponent {
    fn default() -> Self {
        Self {
            shape: ColliderShape::default(),
            sensor: false,
            layer: 0,
            collision_group: 0,
            collision_mask: u32::MAX,
            material: None,
            material_override: None,
            local_transform: Transform::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JointKind {
    #[default]
    Fixed,
    Distance,
    Hinge,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JointComponent {
    pub joint_type: JointKind,
    pub connected_entity: Option<EntityId>,
    pub anchor: Vec3,
    pub axis: Vec3,
    pub limits: Option<[Real; 2]>,
    pub collide_connected: bool,
}

impl Default for JointComponent {
    fn default() -> Self {
        Self {
            joint_type: JointKind::Fixed,
            connected_entity: None,
            anchor: Vec3::ZERO,
            axis: Vec3::Y,
            limits: None,
            collide_connected: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSkeletonComponent {
    pub skeleton: ResourceHandle<AnimationSkeletonMarker>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlayerComponent {
    pub clip: ResourceHandle<AnimationClipMarker>,
    pub playback_speed: Real,
    pub time_seconds: Real,
    pub weight: Real,
    pub looping: bool,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequencePlayerComponent {
    pub sequence: ResourceHandle<AnimationSequenceMarker>,
    pub playback_speed: Real,
    pub time_seconds: Real,
    pub looping: bool,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationGraphPlayerComponent {
    pub graph: ResourceHandle<AnimationGraphMarker>,
    pub parameters: BTreeMap<String, AnimationParameterValue>,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachinePlayerComponent {
    pub state_machine: ResourceHandle<AnimationStateMachineMarker>,
    pub parameters: BTreeMap<String, AnimationParameterValue>,
    pub active_state: Option<String>,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.4, -1.0, -0.25).normalize_or_zero(),
            color: Vec3::splat(1.0),
            intensity: 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointLight {
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Vec3::splat(1.0),
            intensity: 4.0,
            range: 8.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpotLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: Real,
    pub range: Real,
    pub inner_angle_radians: Real,
    pub outer_angle_radians: Real,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::splat(1.0),
            intensity: 8.0,
            range: 12.0,
            inner_angle_radians: 0.3,
            outer_angle_radians: 0.55,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: EntityId,
    pub name: String,
    pub kind: NodeKind,
    pub parent: Option<EntityId>,
    pub transform: Transform,
    pub camera: Option<CameraComponent>,
    pub mesh: Option<MeshRenderer>,
    pub directional_light: Option<DirectionalLight>,
    #[serde(default)]
    pub point_light: Option<PointLight>,
    #[serde(default)]
    pub spot_light: Option<SpotLight>,
    pub rigid_body: Option<RigidBodyComponent>,
    pub collider: Option<ColliderComponent>,
    pub joint: Option<JointComponent>,
    pub animation_skeleton: Option<AnimationSkeletonComponent>,
    pub animation_player: Option<AnimationPlayerComponent>,
    pub animation_sequence_player: Option<AnimationSequencePlayerComponent>,
    pub animation_graph_player: Option<AnimationGraphPlayerComponent>,
    pub animation_state_machine_player: Option<AnimationStateMachinePlayerComponent>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeRecord {
    pub id: EntityId,
    pub name: String,
    pub kind: NodeKind,
    pub parent: Option<EntityId>,
    pub transform: Transform,
    pub camera: Option<CameraComponent>,
    pub mesh: Option<MeshRenderer>,
    pub directional_light: Option<DirectionalLight>,
    #[serde(default)]
    pub point_light: Option<PointLight>,
    #[serde(default)]
    pub spot_light: Option<SpotLight>,
    #[serde(default)]
    pub active: bool,
    #[serde(default = "default_render_layer_mask")]
    pub render_layer_mask: u32,
    #[serde(default)]
    pub mobility: Mobility,
    #[serde(default)]
    pub rigid_body: Option<RigidBodyComponent>,
    #[serde(default)]
    pub collider: Option<ColliderComponent>,
    #[serde(default)]
    pub joint: Option<JointComponent>,
    #[serde(default)]
    pub animation_skeleton: Option<AnimationSkeletonComponent>,
    #[serde(default)]
    pub animation_player: Option<AnimationPlayerComponent>,
    #[serde(default)]
    pub animation_sequence_player: Option<AnimationSequencePlayerComponent>,
    #[serde(default)]
    pub animation_graph_player: Option<AnimationGraphPlayerComponent>,
    #[serde(default)]
    pub animation_state_machine_player: Option<AnimationStateMachinePlayerComponent>,
}

pub const fn default_render_layer_mask() -> u32 {
    0x0000_0001
}
