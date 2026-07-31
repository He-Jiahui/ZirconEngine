use crate::core::framework::scene::physics::{
    PhysicsCcdMode, PhysicsJointConstraintMetadata, PhysicsMassProperties, PhysicsMaterialMetadata,
    PhysicsSkeletonJointBinding, PhysicsSleepPolicy,
};
use crate::core::math::{Real, Transform, Vec3};
use crate::core::resource::{AssetReference, PhysicsMaterialMarker, ResourceHandle};
use crate::scene::EntityId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidBodyType {
    Static,
    #[default]
    Dynamic,
    Kinematic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, zircon_reflect_derive::ZrReflect)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::scene::components::RigidBodyComponent",
    script_visibility = "public",
    field(
        name = "mass_properties_mode",
        value_type_path = "Enum",
        editor_hint = "Enum",
        read = "super::reflection::rigid_body::read_mass_properties_mode",
        write = "super::reflection::rigid_body::write_mass_properties_mode"
    ),
    field(
        name = "mass_density",
        value_type_path = "Scalar",
        editor_hint = "Scalar",
        read = "super::reflection::rigid_body::read_mass_density",
        write = "super::reflection::rigid_body::write_mass_density"
    )
)]
pub struct RigidBodyComponent {
    #[zr_reflect(
        value_type_path = "Enum",
        editor_hint = "Enum",
        read = "super::reflection::rigid_body::read_body_type",
        readonly
    )]
    pub body_type: RigidBodyType,
    pub mass: Real,
    #[serde(default)]
    #[zr_reflect(skip)]
    pub mass_properties: PhysicsMassProperties,
    #[serde(default)]
    #[zr_reflect(
        value_type_path = "Vec3",
        editor_hint = "Vec3",
        read = "super::reflection::rigid_body::read_linear_velocity",
        readonly
    )]
    pub linear_velocity: Vec3,
    #[serde(default)]
    #[zr_reflect(
        value_type_path = "Vec3",
        editor_hint = "Vec3",
        read = "super::reflection::rigid_body::read_angular_velocity",
        readonly
    )]
    pub angular_velocity: Vec3,
    pub linear_damping: Real,
    pub angular_damping: Real,
    pub gravity_scale: Real,
    #[serde(default)]
    #[zr_reflect(
        value_type_path = "Enum",
        editor_hint = "Enum",
        read = "super::reflection::rigid_body::read_ccd_mode",
        write = "super::reflection::rigid_body::write_ccd_mode"
    )]
    pub ccd_mode: PhysicsCcdMode,
    #[serde(default)]
    #[zr_reflect(
        value_type_path = "Enum",
        editor_hint = "Enum",
        read = "super::reflection::rigid_body::read_sleep_policy",
        write = "super::reflection::rigid_body::write_sleep_policy"
    )]
    pub sleep_policy: PhysicsSleepPolicy,
    #[zr_reflect(
        value_type_path = "List<Bool>",
        editor_hint = "None",
        read = "super::reflection::rigid_body::read_lock_translation",
        readonly
    )]
    pub lock_translation: [bool; 3],
    #[zr_reflect(
        value_type_path = "List<Bool>",
        editor_hint = "None",
        read = "super::reflection::rigid_body::read_lock_rotation",
        readonly
    )]
    pub lock_rotation: [bool; 3],
}

impl Default for RigidBodyComponent {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            mass_properties: PhysicsMassProperties::default(),
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            ccd_mode: PhysicsCcdMode::Disabled,
            sleep_policy: PhysicsSleepPolicy::Allow,
            lock_translation: [false; 3],
            lock_rotation: [false; 3],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ColliderShape {
    Box {
        half_extents: Vec3,
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
        points: Vec<Vec3>,
    },
    TriangleMesh {
        mesh: AssetReference,
    },
    HeightField {
        resolution: [u32; 2],
        heights: AssetReference,
    },
    Compound {
        children: Vec<(Transform, Box<ColliderShape>)>,
    },
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
