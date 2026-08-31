use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneRigidBodyAsset {
    body_type: crate::asset::SceneRigidBodyTypeAsset,
    mass: crate::core::math::Real,
    mass_properties: ArtifactCachePhysicsMassProperties,
    linear_velocity: [crate::core::math::Real; 3],
    angular_velocity: [crate::core::math::Real; 3],
    linear_damping: crate::core::math::Real,
    angular_damping: crate::core::math::Real,
    gravity_scale: crate::core::math::Real,
    ccd_mode: crate::core::framework::scene::physics::PhysicsCcdMode,
    sleep_policy: crate::core::framework::scene::physics::PhysicsSleepPolicy,
    lock_translation: [bool; 3],
    lock_rotation: [bool; 3],
}

impl From<&crate::asset::SceneRigidBodyAsset> for ArtifactCacheSceneRigidBodyAsset {
    fn from(asset: &crate::asset::SceneRigidBodyAsset) -> Self {
        Self {
            body_type: asset.body_type,
            mass: asset.mass,
            mass_properties: asset.mass_properties.into(),
            linear_velocity: asset.linear_velocity,
            angular_velocity: asset.angular_velocity,
            linear_damping: asset.linear_damping,
            angular_damping: asset.angular_damping,
            gravity_scale: asset.gravity_scale,
            ccd_mode: asset.ccd_mode,
            sleep_policy: asset.sleep_policy,
            lock_translation: asset.lock_translation,
            lock_rotation: asset.lock_rotation,
        }
    }
}

impl ArtifactCacheSceneRigidBodyAsset {
    pub(super) fn into_asset(self) -> crate::asset::SceneRigidBodyAsset {
        crate::asset::SceneRigidBodyAsset {
            body_type: self.body_type,
            mass: self.mass,
            mass_properties: self.mass_properties.into(),
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            linear_damping: self.linear_damping,
            angular_damping: self.angular_damping,
            gravity_scale: self.gravity_scale,
            ccd_mode: self.ccd_mode,
            sleep_policy: self.sleep_policy,
            lock_translation: self.lock_translation,
            lock_rotation: self.lock_rotation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCachePhysicsMassProperties {
    Explicit {
        inertia_tensor: Option<[[crate::core::math::Real; 3]; 3]>,
    },
    AutoFromShape {
        density: crate::core::math::Real,
    },
}

impl From<crate::core::framework::scene::physics::PhysicsMassProperties>
    for ArtifactCachePhysicsMassProperties
{
    fn from(properties: crate::core::framework::scene::physics::PhysicsMassProperties) -> Self {
        match properties {
            crate::core::framework::scene::physics::PhysicsMassProperties::Explicit {
                inertia_tensor,
            } => Self::Explicit { inertia_tensor },
            crate::core::framework::scene::physics::PhysicsMassProperties::AutoFromShape {
                density,
            } => Self::AutoFromShape { density },
        }
    }
}

impl From<ArtifactCachePhysicsMassProperties>
    for crate::core::framework::scene::physics::PhysicsMassProperties
{
    fn from(properties: ArtifactCachePhysicsMassProperties) -> Self {
        match properties {
            ArtifactCachePhysicsMassProperties::Explicit { inertia_tensor } => {
                Self::Explicit { inertia_tensor }
            }
            ArtifactCachePhysicsMassProperties::AutoFromShape { density } => {
                Self::AutoFromShape { density }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneColliderAsset {
    shape: ArtifactCacheSceneColliderShapeAsset,
    sensor: bool,
    layer: u32,
    collision_group: u32,
    collision_mask: u32,
    material: Option<AssetReference>,
    material_override: Option<crate::core::framework::scene::physics::PhysicsMaterialMetadata>,
    local_transform: crate::asset::TransformAsset,
}

impl From<&crate::asset::SceneColliderAsset> for ArtifactCacheSceneColliderAsset {
    fn from(asset: &crate::asset::SceneColliderAsset) -> Self {
        Self {
            shape: ArtifactCacheSceneColliderShapeAsset::from(&asset.shape),
            sensor: asset.sensor,
            layer: asset.layer,
            collision_group: asset.collision_group,
            collision_mask: asset.collision_mask,
            material: asset.material.clone(),
            material_override: asset.material_override.clone(),
            local_transform: asset.local_transform,
        }
    }
}

impl ArtifactCacheSceneColliderAsset {
    pub(super) fn into_asset(self) -> crate::asset::SceneColliderAsset {
        crate::asset::SceneColliderAsset {
            shape: self.shape.into_asset(),
            sensor: self.sensor,
            layer: self.layer,
            collision_group: self.collision_group,
            collision_mask: self.collision_mask,
            material: self.material,
            material_override: self.material_override,
            local_transform: self.local_transform,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheSceneColliderShapeAsset {
    Box {
        half_extents: [crate::core::math::Real; 3],
    },
    Sphere {
        radius: crate::core::math::Real,
    },
    Capsule {
        radius: crate::core::math::Real,
        half_height: crate::core::math::Real,
    },
    Cylinder {
        radius: crate::core::math::Real,
        half_height: crate::core::math::Real,
    },
    ConvexHull {
        points: Vec<[crate::core::math::Real; 3]>,
    },
    TriangleMesh {
        mesh: crate::asset::AssetReference,
    },
    HeightField {
        resolution: [u32; 2],
        heights: crate::asset::AssetReference,
    },
    Compound {
        children: Vec<(
            crate::asset::TransformAsset,
            Box<ArtifactCacheSceneColliderShapeAsset>,
        )>,
    },
}

impl From<&crate::asset::SceneColliderShapeAsset> for ArtifactCacheSceneColliderShapeAsset {
    fn from(shape: &crate::asset::SceneColliderShapeAsset) -> Self {
        match shape {
            crate::asset::SceneColliderShapeAsset::Box { half_extents } => Self::Box {
                half_extents: *half_extents,
            },
            crate::asset::SceneColliderShapeAsset::Sphere { radius } => {
                Self::Sphere { radius: *radius }
            }
            crate::asset::SceneColliderShapeAsset::Capsule {
                radius,
                half_height,
            } => Self::Capsule {
                radius: *radius,
                half_height: *half_height,
            },
            crate::asset::SceneColliderShapeAsset::Cylinder {
                radius,
                half_height,
            } => Self::Cylinder {
                radius: *radius,
                half_height: *half_height,
            },
            crate::asset::SceneColliderShapeAsset::ConvexHull { points } => Self::ConvexHull {
                points: points.clone(),
            },
            crate::asset::SceneColliderShapeAsset::TriangleMesh { mesh } => {
                Self::TriangleMesh { mesh: mesh.clone() }
            }
            crate::asset::SceneColliderShapeAsset::HeightField {
                resolution,
                heights,
            } => Self::HeightField {
                resolution: *resolution,
                heights: heights.clone(),
            },
            crate::asset::SceneColliderShapeAsset::Compound { children } => Self::Compound {
                children: children
                    .iter()
                    .map(|(transform, shape)| {
                        (
                            *transform,
                            Box::new(ArtifactCacheSceneColliderShapeAsset::from(shape.as_ref())),
                        )
                    })
                    .collect(),
            },
        }
    }
}

impl ArtifactCacheSceneColliderShapeAsset {
    fn into_asset(self) -> crate::asset::SceneColliderShapeAsset {
        match self {
            Self::Box { half_extents } => {
                crate::asset::SceneColliderShapeAsset::Box { half_extents }
            }
            Self::Sphere { radius } => crate::asset::SceneColliderShapeAsset::Sphere { radius },
            Self::Capsule {
                radius,
                half_height,
            } => crate::asset::SceneColliderShapeAsset::Capsule {
                radius,
                half_height,
            },
            Self::Cylinder {
                radius,
                half_height,
            } => crate::asset::SceneColliderShapeAsset::Cylinder {
                radius,
                half_height,
            },
            Self::ConvexHull { points } => {
                crate::asset::SceneColliderShapeAsset::ConvexHull { points }
            }
            Self::TriangleMesh { mesh } => {
                crate::asset::SceneColliderShapeAsset::TriangleMesh { mesh }
            }
            Self::HeightField {
                resolution,
                heights,
            } => crate::asset::SceneColliderShapeAsset::HeightField {
                resolution,
                heights,
            },
            Self::Compound { children } => crate::asset::SceneColliderShapeAsset::Compound {
                children: children
                    .into_iter()
                    .map(|(transform, shape)| (transform, Box::new(shape.into_asset())))
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneJointAsset {
    joint_type: crate::asset::SceneJointKindAsset,
    connected_entity: Option<u64>,
    anchor: [crate::core::math::Real; 3],
    axis: [crate::core::math::Real; 3],
    limits: Option<[crate::core::math::Real; 2]>,
    collide_connected: bool,
    constraint: ArtifactCachePhysicsJointConstraintMetadata,
    skeleton_binding: Option<crate::core::framework::scene::physics::PhysicsSkeletonJointBinding>,
}

impl From<&crate::asset::SceneJointAsset> for ArtifactCacheSceneJointAsset {
    fn from(asset: &crate::asset::SceneJointAsset) -> Self {
        Self {
            joint_type: asset.joint_type,
            connected_entity: asset.connected_entity,
            anchor: asset.anchor,
            axis: asset.axis,
            limits: asset.limits,
            collide_connected: asset.collide_connected,
            constraint: ArtifactCachePhysicsJointConstraintMetadata::from(&asset.constraint),
            skeleton_binding: asset.skeleton_binding.clone(),
        }
    }
}

impl ArtifactCacheSceneJointAsset {
    pub(super) fn into_asset(self) -> crate::asset::SceneJointAsset {
        crate::asset::SceneJointAsset {
            joint_type: self.joint_type,
            connected_entity: self.connected_entity,
            anchor: self.anchor,
            axis: self.axis,
            limits: self.limits,
            collide_connected: self.collide_connected,
            constraint: self.constraint.into(),
            skeleton_binding: self.skeleton_binding,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCachePhysicsJointConstraintMetadata {
    linear_limits: [Option<[crate::core::math::Real; 2]>; 3],
    angular_limits: [Option<[crate::core::math::Real; 2]>; 3],
    linear_drives: [crate::core::framework::scene::physics::PhysicsJointDrive; 3],
    angular_drives: [crate::core::framework::scene::physics::PhysicsJointDrive; 3],
    break_force: Option<crate::core::math::Real>,
    break_torque: Option<crate::core::math::Real>,
    projection_linear_tolerance: Option<crate::core::math::Real>,
    projection_angular_tolerance: Option<crate::core::math::Real>,
}

impl From<&crate::core::framework::scene::physics::PhysicsJointConstraintMetadata>
    for ArtifactCachePhysicsJointConstraintMetadata
{
    fn from(
        metadata: &crate::core::framework::scene::physics::PhysicsJointConstraintMetadata,
    ) -> Self {
        Self {
            linear_limits: metadata.linear_limits,
            angular_limits: metadata.angular_limits,
            linear_drives: metadata.linear_drives,
            angular_drives: metadata.angular_drives,
            break_force: metadata.break_force,
            break_torque: metadata.break_torque,
            projection_linear_tolerance: metadata.projection_linear_tolerance,
            projection_angular_tolerance: metadata.projection_angular_tolerance,
        }
    }
}

impl From<ArtifactCachePhysicsJointConstraintMetadata>
    for crate::core::framework::scene::physics::PhysicsJointConstraintMetadata
{
    fn from(metadata: ArtifactCachePhysicsJointConstraintMetadata) -> Self {
        Self {
            linear_limits: metadata.linear_limits,
            angular_limits: metadata.angular_limits,
            linear_drives: metadata.linear_drives,
            angular_drives: metadata.angular_drives,
            break_force: metadata.break_force,
            break_torque: metadata.break_torque,
            projection_linear_tolerance: metadata.projection_linear_tolerance,
            projection_angular_tolerance: metadata.projection_angular_tolerance,
        }
    }
}
