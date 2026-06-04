use zircon_runtime::core::framework::physics::PhysicsRayCastHit;
use zircon_runtime::core::math::{Real, Vec3};

use super::super::geometry::{normalized_offset_or_zero, ray_hit_position};
use super::quadratic::{ray_distance_to_real, ray_sphere_quadratic_distances};

pub(super) fn ray_cast_sphere(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    entity: u64,
    center: Vec3,
    radius: Real,
) -> Option<PhysicsRayCastHit> {
    let [near_distance, far_distance] =
        ray_sphere_quadratic_distances(origin, direction, center, radius)?;
    let distance_f64 = if near_distance >= 0.0 {
        near_distance
    } else {
        far_distance
    };
    let distance = ray_distance_to_real(distance_f64, max_distance)?;

    let position = ray_hit_position(origin, direction, distance)?;
    let normal = normalized_offset_or_zero(position, center);
    Some(PhysicsRayCastHit {
        entity,
        distance,
        position: position.to_array(),
        normal: normal.to_array(),
    })
}
