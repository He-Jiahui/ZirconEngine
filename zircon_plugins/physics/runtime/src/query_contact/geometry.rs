use zircon_runtime::core::framework::physics::{PhysicsColliderShape, PhysicsColliderSyncState};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};

pub(super) fn collider_geometry_is_valid(collider: &PhysicsColliderSyncState) -> bool {
    if !transform_is_finite(collider.transform) {
        return false;
    }

    match collider.shape {
        PhysicsColliderShape::Box { half_extents } => box_geometry_is_valid(half_extents),
        PhysicsColliderShape::Sphere { radius } => positive_finite(radius),
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => capsule_geometry_is_valid(radius, half_height),
    }
}

fn transform_is_finite(value: Transform) -> bool {
    vec3_is_finite(value.translation)
        && quat_is_finite(value.rotation)
        && vec3_is_finite(value.scale)
}

fn quat_is_finite(value: Quat) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite() && value.w.is_finite()
}

pub(super) fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

pub(super) fn midpoint(left: Vec3, right: Vec3) -> Vec3 {
    left + (right - left) * 0.5
}

pub(super) fn collider_aabb(collider: &PhysicsColliderSyncState) -> Option<(Vec3, Vec3)> {
    let center = collider.transform.translation;
    let scale = collider.transform.scale.abs();
    let half_extents = match collider.shape {
        PhysicsColliderShape::Box { half_extents } => scaled_box_half_extents(half_extents, scale)?,
        PhysicsColliderShape::Sphere { radius } => {
            let scaled_radius = radius * max_abs_scale(collider.transform.scale);
            if !positive_finite(scaled_radius) {
                return None;
            }
            Vec3::splat(scaled_radius)
        }
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => {
            let scaled_radius_x = radius * scale.x;
            let scaled_radius_z = radius * scale.z;
            let scaled_half_height = (radius + half_height) * scale.y;
            let half_extents = Vec3::new(scaled_radius_x, scaled_half_height, scaled_radius_z);
            if !vec3_is_finite(half_extents) {
                return None;
            }
            half_extents
        }
    };
    finite_aabb_bounds(center, half_extents)
}

pub(super) fn max_abs_scale(scale: Vec3) -> Real {
    scale.x.abs().max(scale.y.abs()).max(scale.z.abs())
}

pub(super) fn positive_finite(value: Real) -> bool {
    value.is_finite() && value > 0.0
}

pub(super) fn capsule_geometry_is_valid(radius: Real, half_height: Real) -> bool {
    positive_finite(radius) && half_height.is_finite() && half_height >= 0.0
}

pub(super) fn box_geometry_is_valid(half_extents: [Real; 3]) -> bool {
    half_extents
        .iter()
        .all(|extent| extent.is_finite() && *extent >= 0.0)
}

pub(super) fn scaled_box_half_extents(half_extents: [Real; 3], scale: Vec3) -> Option<Vec3> {
    let scaled_half_extents = Vec3::from_array(half_extents) * scale.abs();
    vec3_is_finite(scaled_half_extents).then_some(scaled_half_extents)
}

pub(super) fn finite_aabb_bounds(center: Vec3, half_extents: Vec3) -> Option<(Vec3, Vec3)> {
    let min = center - half_extents;
    let max = center + half_extents;
    (vec3_is_finite(min) && vec3_is_finite(max)).then_some((min, max))
}

pub(super) fn point_interval_gap(point: f64, min: f64, max: f64) -> f64 {
    if point < min {
        min - point
    } else if point > max {
        point - max
    } else {
        0.0
    }
}

pub(super) fn interval_interval_gap(
    left_min: f64,
    left_max: f64,
    right_min: f64,
    right_max: f64,
) -> f64 {
    if left_max < right_min {
        right_min - left_max
    } else if right_max < left_min {
        left_min - right_max
    } else {
        0.0
    }
}

pub(super) fn normalized_offset_or_zero(point: Vec3, center: Vec3) -> Vec3 {
    let dx = f64::from(point.x) - f64::from(center.x);
    let dy = f64::from(point.y) - f64::from(center.y);
    let dz = f64::from(point.z) - f64::from(center.z);
    let length = (dx * dx + dy * dy + dz * dz).sqrt();
    if !length.is_finite() || length <= f64::EPSILON {
        return Vec3::ZERO;
    }
    Vec3::new(
        (dx / length) as Real,
        (dy / length) as Real,
        (dz / length) as Real,
    )
}

pub(super) fn normalized_xz_offset_or_zero(point: Vec3, center: Vec3) -> Vec3 {
    let dx = f64::from(point.x) - f64::from(center.x);
    let dz = f64::from(point.z) - f64::from(center.z);
    let length = (dx * dx + dz * dz).sqrt();
    if !length.is_finite() || length <= f64::EPSILON {
        return Vec3::ZERO;
    }
    Vec3::new((dx / length) as Real, 0.0, (dz / length) as Real)
}

pub(super) fn ray_hit_position(origin: Vec3, direction: Vec3, distance: Real) -> Option<Vec3> {
    let position = origin + direction * distance;
    vec3_is_finite(position).then_some(position)
}
