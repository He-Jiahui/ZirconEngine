use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::render::{
    CorePipelineKind, MaterialPropertyOverrideBlock, ProjectionMode, RenderCameraClearColor,
    RenderCameraTarget, RenderMaterialAlphaMode, RenderViewportRect, DEFAULT_CAMERA_EXPOSURE_EV100,
    DEFAULT_CAMERA_MSAA_SAMPLES, DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::framework::scene::physics::{
    PhysicsJointConstraintMetadata, PhysicsMaterialMetadata, PhysicsSkeletonJointBinding,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Mat4, Real, Transform, Vec3, Vec4};
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, MaterialMarker, MeshMarker, ModelMarker, PhysicsMaterialMarker,
    ResourceHandle, ResourceId,
};
use serde::{Deserialize, Serialize};

use super::{Mesh2dComponent, Sprite2dComponent};
use crate::scene::EntityId;

mod lighting;
mod post_process;

pub use self::lighting::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight};
pub use self::post_process::{PostProcessSettingsComponent, PostProcessVolumeComponent};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Empty,
    Camera,
    Cube,
    Mesh,
    AmbientLight,
    DirectionalLight,
    PointLight,
    RectLight,
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
    /// Selects Core2d or Core3d without constraining perspective/orthographic projection.
    #[serde(default)]
    pub core_pipeline: CorePipelineKind,
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
    pub target: RenderCameraTarget,
    #[serde(default)]
    pub viewport: Option<RenderViewportRect>,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default = "default_camera_exposure_ev100")]
    pub exposure_ev100: Real,
    #[serde(default)]
    pub clear_color: RenderCameraClearColor,
    #[serde(default = "default_camera_msaa_samples")]
    pub msaa_samples: u32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            core_pipeline: CorePipelineKind::Core3d,
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: default_camera_fov_y_radians(),
            ortho_size: default_camera_ortho_size(),
            z_near: default_camera_z_near(),
            z_far: default_camera_z_far(),
            target: RenderCameraTarget::default(),
            viewport: None,
            order: 0,
            is_active: true,
            hdr: false,
            exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
            clear_color: RenderCameraClearColor::default(),
            msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshRendererPrimitiveBinding {
    pub mesh: ResourceHandle<MeshMarker>,
    pub material: ResourceHandle<MaterialMarker>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshRendererLodLevel {
    #[serde(default)]
    pub min_distance: Real,
    pub model: ResourceHandle<ModelMarker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    pub material: ResourceHandle<MaterialMarker>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primitives: Vec<MeshRendererPrimitiveBinding>,
}

impl MeshRendererLodLevel {
    pub fn from_handles(
        min_distance: Real,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self {
            min_distance,
            model,
            mesh: None,
            material,
            primitives: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshRenderer {
    pub model: ResourceHandle<ModelMarker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    pub material: ResourceHandle<MaterialMarker>,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub render_queue: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub material_queue: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub order_in_layer: i32,
    #[serde(default, skip_serializing_if = "is_zero_real")]
    pub depth_bias: Real,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morph_weights: Vec<Real>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primitives: Vec<MeshRendererPrimitiveBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lods: Vec<MeshRendererLodLevel>,
    #[serde(
        default,
        skip_serializing_if = "MaterialPropertyOverrideBlock::is_empty"
    )]
    pub material_property_overrides: MaterialPropertyOverrideBlock,
    pub tint: Vec4,
    #[serde(default)]
    pub material_alpha_mode: RenderMaterialAlphaMode,
}

impl MeshRenderer {
    pub fn from_handles(
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self {
            model,
            mesh: None,
            material,
            render_queue: 0,
            material_queue: 0,
            order_in_layer: 0,
            depth_bias: 0.0,
            morph_weights: Vec::new(),
            primitives: Vec::new(),
            lods: Vec::new(),
            material_property_overrides: MaterialPropertyOverrideBlock::default(),
            tint: Vec4::ONE,
            material_alpha_mode: RenderMaterialAlphaMode::Opaque,
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
    #[serde(default)]
    pub linear_velocity: Vec3,
    #[serde(default)]
    pub angular_velocity: Vec3,
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
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
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
    Slider,
    ConeTwist,
    Generic6Dof,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JointComponent {
    pub joint_type: JointKind,
    pub connected_entity: Option<EntityId>,
    pub anchor: Vec3,
    pub axis: Vec3,
    pub limits: Option<[Real; 2]>,
    pub collide_connected: bool,
    #[serde(default)]
    pub constraint: PhysicsJointConstraintMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skeleton_binding: Option<PhysicsSkeletonJointBinding>,
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
            constraint: PhysicsJointConstraintMetadata::default(),
            skeleton_binding: None,
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
pub struct SceneNode {
    pub id: EntityId,
    pub name: String,
    pub kind: NodeKind,
    pub parent: Option<EntityId>,
    pub transform: Transform,
    pub camera: Option<CameraComponent>,
    pub mesh: Option<MeshRenderer>,
    #[serde(default)]
    pub sprite_2d: Option<Sprite2dComponent>,
    #[serde(default)]
    pub mesh_2d: Option<Mesh2dComponent>,
    #[serde(default)]
    pub ambient_light: Option<AmbientLight>,
    pub directional_light: Option<DirectionalLight>,
    #[serde(default)]
    pub point_light: Option<PointLight>,
    #[serde(default)]
    pub rect_light: Option<RectLight>,
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
    #[serde(default)]
    pub sprite_2d: Option<Sprite2dComponent>,
    #[serde(default)]
    pub mesh_2d: Option<Mesh2dComponent>,
    #[serde(default)]
    pub ambient_light: Option<AmbientLight>,
    pub directional_light: Option<DirectionalLight>,
    #[serde(default)]
    pub point_light: Option<PointLight>,
    #[serde(default)]
    pub rect_light: Option<RectLight>,
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
    DEFAULT_RENDER_LAYER_MASK
}

const fn default_true() -> bool {
    true
}

fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

fn is_zero_real(value: &Real) -> bool {
    *value == 0.0
}

const fn default_camera_ortho_size() -> Real {
    5.0
}

fn default_camera_fov_y_radians() -> Real {
    60.0_f32.to_radians()
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
