mod aabb;
mod capsule;
mod quadratic;
mod sphere;

use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsRayCastHit,
};
use zircon_runtime::core::math::{Real, Vec3};

use super::geometry::{
    collider_geometry_is_valid, finite_aabb_bounds, max_abs_scale, positive_finite,
    scaled_box_half_extents,
};

pub(super) fn ray_cast_collider(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    collider: &PhysicsColliderSyncState,
) -> Option<PhysicsRayCastHit> {
    if !collider_geometry_is_valid(collider) {
        return None;
    }

    match collider.shape {
        PhysicsColliderShape::Box { half_extents } => {
            let scaled_half_extents =
                scaled_box_half_extents(half_extents, collider.transform.scale)?;
            let center = collider.transform.translation;
            let (min, max) = finite_aabb_bounds(center, scaled_half_extents)?;
            aabb::ray_cast_aabb(origin, direction, max_distance, collider.entity, min, max)
        }
        PhysicsColliderShape::Sphere { radius } => {
            let scaled_radius = radius * max_abs_scale(collider.transform.scale);
            if !positive_finite(scaled_radius) {
                return None;
            }
            sphere::ray_cast_sphere(
                origin,
                direction,
                max_distance,
                collider.entity,
                collider.transform.translation,
                scaled_radius,
            )
        }
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => {
            let scale = collider.transform.scale.abs();
            let scaled_radius = radius * scale.x.max(scale.z);
            let scaled_half_height = half_height * scale.y;
            if !positive_finite(scaled_radius) || !scaled_half_height.is_finite() {
                return None;
            }
            capsule::ray_cast_capsule_y(
                origin,
                direction,
                max_distance,
                collider.entity,
                collider.transform.translation,
                scaled_radius,
                scaled_half_height,
            )
        }
    }
}
