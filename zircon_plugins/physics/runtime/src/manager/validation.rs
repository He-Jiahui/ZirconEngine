use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsColliderShape, PhysicsColliderSyncState,
        PhysicsJointSyncState, PhysicsMaterialSyncState,
    },
    scene::physics::{
        PhysicsJointConstraintMetadata, PhysicsJointDrive, PhysicsMassProperties,
        PhysicsMaterialMetadata, PhysicsSkeletonJointBinding,
    },
};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::scene::components::{ColliderShape, JointComponent, RigidBodyComponent};

pub(crate) fn rigid_body_step_input_is_finite(rigid_body: &RigidBodyComponent) -> bool {
    vec3_is_finite(rigid_body.linear_velocity)
        && vec3_is_finite(rigid_body.angular_velocity)
        && rigid_body.linear_damping.is_finite()
        && rigid_body.angular_damping.is_finite()
        && rigid_body.gravity_scale.is_finite()
}

pub(super) fn rigid_body_sync_input_is_finite(rigid_body: &RigidBodyComponent) -> bool {
    rigid_body.mass_properties.is_valid()
        && (matches!(
            rigid_body.mass_properties,
            PhysicsMassProperties::AutoFromShape { .. }
        ) || (rigid_body.mass.is_finite() && rigid_body.mass > 0.0))
        && rigid_body_step_input_is_finite(rigid_body)
}

pub(super) fn joint_sync_input_is_finite(joint: &JointComponent) -> bool {
    vec3_is_finite(joint.anchor)
        && vec3_is_finite(joint.axis)
        && joint.limits.is_none_or(physics_limit_range_is_valid)
        && physics_joint_constraint_metadata_is_valid(&joint.constraint)
        && joint
            .skeleton_binding
            .as_ref()
            .is_none_or(physics_skeleton_joint_binding_is_valid)
}

pub(super) fn collider_shape_sync_input_is_valid(shape: &ColliderShape) -> bool {
    match shape {
        ColliderShape::Box { half_extents } => {
            half_extents.x.is_finite()
                && half_extents.x >= 0.0
                && half_extents.y.is_finite()
                && half_extents.y >= 0.0
                && half_extents.z.is_finite()
                && half_extents.z >= 0.0
        }
        ColliderShape::Sphere { radius } => radius.is_finite() && *radius > 0.0,
        ColliderShape::Capsule {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height >= 0.0,
        ColliderShape::Cylinder {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height > 0.0,
        ColliderShape::ConvexHull { points } => {
            points.len() >= 4 && points.iter().all(|point| vec3_is_finite(*point))
        }
        ColliderShape::TriangleMesh { .. } => true,
        ColliderShape::HeightField { resolution, .. } => resolution[0] >= 2 && resolution[1] >= 2,
        ColliderShape::Compound { children } => {
            !children.is_empty()
                && children.iter().all(|(transform, child)| {
                    transform_is_finite(*transform)
                        && transform.scale == Vec3::ONE
                        && collider_shape_sync_input_is_valid(child)
                })
        }
    }
}

pub(super) fn collider_layer_sync_input_is_valid(layer: u32) -> bool {
    layer < u32::BITS
}

pub(super) fn material_metadata_sync_input_is_finite(material: &PhysicsMaterialMetadata) -> bool {
    material.static_friction.is_finite()
        && material.dynamic_friction.is_finite()
        && material.restitution.is_finite()
}

pub(crate) fn transform_is_finite(transform: Transform) -> bool {
    vec3_is_finite(transform.translation)
        && quat_is_finite(transform.rotation)
        && vec3_is_finite(transform.scale)
}

fn quat_is_finite(value: zircon_runtime::core::math::Quat) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite() && value.w.is_finite()
}

fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

pub(super) fn array3_is_finite(value: [Real; 3]) -> bool {
    value[0].is_finite() && value[1].is_finite() && value[2].is_finite()
}

fn array2_is_finite(value: [Real; 2]) -> bool {
    value[0].is_finite() && value[1].is_finite()
}

fn physics_limit_range_is_valid(value: [Real; 2]) -> bool {
    array2_is_finite(value) && value[0] <= value[1]
}

fn physics_joint_constraint_metadata_is_valid(constraint: &PhysicsJointConstraintMetadata) -> bool {
    constraint
        .linear_limits
        .iter()
        .all(|limit| limit.is_none_or(physics_limit_range_is_valid))
        && constraint
            .angular_limits
            .iter()
            .all(|limit| limit.is_none_or(physics_limit_range_is_valid))
        && constraint
            .linear_drives
            .iter()
            .chain(constraint.angular_drives.iter())
            .all(physics_joint_drive_is_valid)
        && optional_non_negative_finite(constraint.break_force)
        && optional_non_negative_finite(constraint.break_torque)
        && optional_non_negative_finite(constraint.projection_linear_tolerance)
        && optional_non_negative_finite(constraint.projection_angular_tolerance)
}

fn physics_joint_drive_is_valid(drive: &PhysicsJointDrive) -> bool {
    drive.target_position.is_finite()
        && drive.target_velocity.is_finite()
        && drive.stiffness.is_finite()
        && drive.stiffness >= 0.0
        && drive.damping.is_finite()
        && drive.damping >= 0.0
        && drive.max_force.is_finite()
        && drive.max_force >= 0.0
}

fn optional_non_negative_finite(value: Option<Real>) -> bool {
    value.is_none_or(|value| value.is_finite() && value >= 0.0)
}

fn physics_skeleton_joint_binding_is_valid(binding: &PhysicsSkeletonJointBinding) -> bool {
    !binding.bone_path.trim().is_empty()
        && binding
            .parent_bone_path
            .as_deref()
            .is_none_or(|path| !path.trim().is_empty())
}

pub(super) fn normalized_ray_direction(direction: [Real; 3]) -> Option<Vec3> {
    let [x, y, z] = direction.map(f64::from);
    let length = (x * x + y * y + z * z).sqrt();
    if !length.is_finite() || length <= f64::EPSILON {
        return None;
    }
    Some(Vec3::new(
        (x / length) as Real,
        (y / length) as Real,
        (z / length) as Real,
    ))
}

pub(super) fn physics_body_sync_state_is_valid(body: &PhysicsBodySyncState) -> bool {
    transform_is_finite(body.transform)
        && body.mass.is_finite()
        && body.mass > 0.0
        && body.mass_properties.is_valid()
        && array3_is_finite(body.linear_velocity)
        && array3_is_finite(body.angular_velocity)
        && body.linear_damping.is_finite()
        && body.angular_damping.is_finite()
        && body.gravity_scale.is_finite()
}

pub(super) fn physics_collider_sync_state_is_valid(collider: &PhysicsColliderSyncState) -> bool {
    transform_is_finite(collider.transform)
        && collider_layer_sync_input_is_valid(collider.layer)
        && physics_collider_shape_is_valid(&collider.shape)
        && material_locator_sync_input_is_valid(&collider.material)
        && collider
            .material_override
            .as_ref()
            .is_none_or(material_metadata_sync_input_is_finite)
}

pub(super) fn physics_joint_sync_state_is_valid(joint: &PhysicsJointSyncState) -> bool {
    array3_is_finite(joint.anchor)
        && array3_is_finite(joint.axis)
        && joint.limits.is_none_or(physics_limit_range_is_valid)
        && physics_joint_constraint_metadata_is_valid(&joint.constraint)
        && joint
            .skeleton_binding
            .as_ref()
            .is_none_or(physics_skeleton_joint_binding_is_valid)
}

pub(super) fn physics_material_sync_state_is_valid(material: &PhysicsMaterialSyncState) -> bool {
    material_locator_sync_input_is_valid(&material.locator)
        && material_metadata_sync_input_is_finite(&material.material)
}

pub(super) fn physics_collider_shape_is_valid(shape: &PhysicsColliderShape) -> bool {
    match shape {
        PhysicsColliderShape::Box { half_extents } => {
            array3_is_finite(*half_extents) && half_extents.iter().all(|extent| *extent >= 0.0)
        }
        PhysicsColliderShape::Sphere { radius } => radius.is_finite() && *radius > 0.0,
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height >= 0.0,
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height > 0.0,
        PhysicsColliderShape::ConvexHull { points } => {
            points.len() >= 4 && points.iter().all(|point| array3_is_finite(*point))
        }
        PhysicsColliderShape::TriangleMesh { .. } => true,
        PhysicsColliderShape::HeightField { resolution, .. } => {
            resolution[0] >= 2 && resolution[1] >= 2
        }
        PhysicsColliderShape::Compound { children } => {
            !children.is_empty()
                && children.iter().all(|(transform, child)| {
                    transform.translation.is_finite()
                        && transform.rotation.is_finite()
                        && transform.scale.is_finite()
                        && transform.scale == Vec3::ONE
                        && physics_collider_shape_is_valid(child)
                })
        }
    }
}

pub(super) fn material_locator_sync_input_is_valid(locator: &Option<String>) -> bool {
    locator
        .as_deref()
        .is_none_or(|locator| !locator.trim().is_empty())
}
