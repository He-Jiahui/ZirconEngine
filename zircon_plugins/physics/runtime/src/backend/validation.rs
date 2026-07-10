use zircon_runtime::core::framework::{
    physics::PhysicsColliderShape, scene::physics::PhysicsMaterialMetadata,
};

use super::BodyDesc;

pub(super) fn body_desc_is_valid(desc: &BodyDesc) -> bool {
    let body = &desc.body;
    body.entity == desc.collider.entity
        && body.mass.is_finite()
        && body.mass > 0.0
        && body.linear_velocity.iter().all(|value| value.is_finite())
        && body.angular_velocity.iter().all(|value| value.is_finite())
        && body.linear_damping.is_finite()
        && body.angular_damping.is_finite()
        && body.gravity_scale.is_finite()
        && body.transform.translation.is_finite()
        && body.transform.rotation.is_finite()
        && body.transform.scale.is_finite()
}

pub(super) fn shape_is_valid(shape: &PhysicsColliderShape) -> bool {
    match shape {
        PhysicsColliderShape::Box { half_extents } => half_extents
            .iter()
            .all(|extent| extent.is_finite() && *extent >= 0.0),
        PhysicsColliderShape::Sphere { radius } => radius.is_finite() && *radius > 0.0,
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height >= 0.0,
    }
}

pub(super) fn material_is_valid(material: &PhysicsMaterialMetadata) -> bool {
    material.static_friction.is_finite()
        && material.dynamic_friction.is_finite()
        && material.restitution.is_finite()
}
