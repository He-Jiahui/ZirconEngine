use crate::asset::AssetReference;
use crate::core::framework::scene::physics::{
    PhysicsCcdMode, PhysicsJointConstraintMetadata, PhysicsMassProperties, PhysicsMaterialMetadata,
    PhysicsSkeletonJointBinding, PhysicsSleepPolicy,
};
use crate::core::math::Real;
use serde::{Deserialize, Serialize};

use super::defaults::{
    default_collision_mask, default_gravity_scale, default_rigid_body_mass, default_vec3_up,
    default_vec3_zero,
};
use super::transform::TransformAsset;

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
    pub mass_properties: PhysicsMassProperties,
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
    #[serde(default)]
    pub ccd_mode: PhysicsCcdMode,
    #[serde(default)]
    pub sleep_policy: PhysicsSleepPolicy,
    #[serde(default)]
    pub lock_translation: [bool; 3],
    #[serde(default)]
    pub lock_rotation: [bool; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneColliderShapeAsset {
    Box {
        half_extents: [Real; 3],
    },
    Sphere {
        radius: Real,
    },
    Capsule {
        radius: Real,
        half_height: Real,
    },
    Cylinder {
        radius: Real,
        half_height: Real,
    },
    ConvexHull {
        points: Vec<[Real; 3]>,
    },
    TriangleMesh {
        mesh: AssetReference,
    },
    HeightField {
        resolution: [u32; 2],
        heights: AssetReference,
    },
    Compound {
        children: Vec<(TransformAsset, Box<SceneColliderShapeAsset>)>,
    },
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
    Slider,
    ConeTwist,
    Generic6Dof,
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
    #[serde(default)]
    pub constraint: PhysicsJointConstraintMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skeleton_binding: Option<PhysicsSkeletonJointBinding>,
}
