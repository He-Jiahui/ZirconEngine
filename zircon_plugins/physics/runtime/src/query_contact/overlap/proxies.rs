use zircon_runtime::core::framework::physics::{PhysicsColliderShape, PhysicsColliderSyncState};
use zircon_runtime::core::math::{Real, Vec3};

use super::super::geometry::{
    box_geometry_is_valid, capsule_geometry_is_valid, finite_aabb_bounds, max_abs_scale,
    positive_finite, scaled_box_half_extents,
};

#[derive(Clone, Copy)]
pub(super) struct SphereOverlapProxy {
    pub(super) center: Vec3,
    pub(super) radius: Real,
}

#[derive(Clone, Copy)]
pub(super) struct BoxOverlapProxy {
    pub(super) min: Vec3,
    pub(super) max: Vec3,
}

#[derive(Clone, Copy)]
pub(super) struct CapsuleOverlapProxy {
    pub(super) center: Vec3,
    pub(super) radius: Real,
    pub(super) half_height: Real,
}

pub(super) fn collider_sphere(collider: &PhysicsColliderSyncState) -> Option<SphereOverlapProxy> {
    let PhysicsColliderShape::Sphere { radius } = collider.shape else {
        return None;
    };
    if !positive_finite(radius) {
        return None;
    }
    let scaled_radius = radius * max_abs_scale(collider.transform.scale);
    if !positive_finite(scaled_radius) {
        return None;
    }
    Some(SphereOverlapProxy {
        center: collider.transform.translation,
        radius: scaled_radius,
    })
}

pub(super) fn collider_box(collider: &PhysicsColliderSyncState) -> Option<BoxOverlapProxy> {
    let PhysicsColliderShape::Box { half_extents } = collider.shape else {
        return None;
    };
    if !box_geometry_is_valid(half_extents) {
        return None;
    }
    let center = collider.transform.translation;
    let scaled_half_extents = scaled_box_half_extents(half_extents, collider.transform.scale)?;
    let (min, max) = finite_aabb_bounds(center, scaled_half_extents)?;
    Some(BoxOverlapProxy { min, max })
}

pub(super) fn collider_capsule_y(
    collider: &PhysicsColliderSyncState,
) -> Option<CapsuleOverlapProxy> {
    let PhysicsColliderShape::Capsule {
        radius,
        half_height,
    } = collider.shape
    else {
        return None;
    };
    if !capsule_geometry_is_valid(radius, half_height) {
        return None;
    }
    let scale = collider.transform.scale.abs();
    let scaled_radius = radius * scale.x.max(scale.z);
    let scaled_half_height = half_height * scale.y;
    if !positive_finite(scaled_radius) || !scaled_half_height.is_finite() {
        return None;
    }
    Some(CapsuleOverlapProxy {
        center: collider.transform.translation,
        radius: scaled_radius,
        half_height: scaled_half_height,
    })
}
