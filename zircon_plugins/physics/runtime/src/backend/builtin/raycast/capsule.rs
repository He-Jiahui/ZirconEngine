use zircon_runtime::core::framework::physics::PhysicsRayCastHit;
use zircon_runtime::core::math::{Real, Vec3};

use super::super::geometry::{
    normalized_offset_or_zero, normalized_xz_offset_or_zero, ray_hit_position,
};
use super::quadratic::{
    ray_distance_to_real, ray_quadratic_distances, ray_sphere_quadratic_distances,
};

pub(super) fn ray_cast_capsule_y(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
    half_height: Real,
) -> Option<PhysicsRayCastHit> {
    if radius <= 0.0 {
        return None;
    }

    [
        ray_cast_capsule_cylinder_y(
            origin,
            direction,
            max_distance,
            entity,
            center,
            radius,
            half_height,
        ),
        ray_cast_capsule_cap_y(
            origin,
            direction,
            max_distance,
            entity,
            center + Vec3::Y * half_height,
            radius,
            center.y + half_height,
            true,
        ),
        ray_cast_capsule_cap_y(
            origin,
            direction,
            max_distance,
            entity,
            center - Vec3::Y * half_height,
            radius,
            center.y - half_height,
            false,
        ),
    ]
    .into_iter()
    .flatten()
    .min_by(|left, right| {
        left.distance
            .partial_cmp(&right.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn ray_cast_capsule_cap_y(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
    boundary_y: Real,
    upper: bool,
) -> Option<PhysicsRayCastHit> {
    ray_sphere_quadratic_distances(origin, direction, center, radius)?
        .into_iter()
        .filter_map(|distance| ray_distance_to_real(distance, max_distance))
        .filter_map(|distance| {
            let position = ray_hit_position(origin, direction, distance)?;
            let on_visible_cap = if upper {
                position.y >= boundary_y
            } else {
                position.y <= boundary_y
            };
            if !on_visible_cap {
                return None;
            }
            let normal = normalized_offset_or_zero(position, center);
            Some(PhysicsRayCastHit {
                entity,
                distance,
                position: position.to_array(),
                normal: normal.to_array(),
            })
        })
        .min_by(|left, right| {
            left.distance
                .partial_cmp(&right.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn ray_cast_capsule_cylinder_y(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
    half_height: Real,
) -> Option<PhysicsRayCastHit> {
    let offset_x = f64::from(origin.x) - f64::from(center.x);
    let offset_z = f64::from(origin.z) - f64::from(center.z);
    let direction_x = f64::from(direction.x);
    let direction_z = f64::from(direction.z);
    let a = direction_x * direction_x + direction_z * direction_z;
    let b = 2.0 * (offset_x * direction_x + offset_z * direction_z);
    let c = offset_x * offset_x + offset_z * offset_z - f64::from(radius).powi(2);
    ray_quadratic_distances(a, b, c)?
        .into_iter()
        .filter_map(|distance| ray_distance_to_real(distance, max_distance))
        .filter_map(|distance| {
            let position = ray_hit_position(origin, direction, distance)?;
            let local_y = position.y - center.y;
            if local_y.abs() > half_height {
                return None;
            }
            let normal = normalized_xz_offset_or_zero(position, center);
            Some(PhysicsRayCastHit {
                entity,
                distance,
                position: position.to_array(),
                normal: normal.to_array(),
            })
        })
        .min_by(|left, right| {
            left.distance
                .partial_cmp(&right.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}
