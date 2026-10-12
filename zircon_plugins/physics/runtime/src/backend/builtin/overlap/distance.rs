use zircon_runtime::core::math::{Real, Vec3};

use super::super::geometry::{interval_interval_gap, point_interval_gap};
use super::proxies::{BoxOverlapProxy, CapsuleOverlapProxy};

pub(super) fn point_distance_squared_lte_radius_sum(
    left_point: Vec3,
    right_point: Vec3,
    left_radius: Real,
    right_radius: Real,
) -> bool {
    let dx = f64::from(left_point.x) - f64::from(right_point.x);
    let dy = f64::from(left_point.y) - f64::from(right_point.y);
    let dz = f64::from(left_point.z) - f64::from(right_point.z);
    let distance_squared = dx * dx + dy * dy + dz * dz;
    distance_squared_lte_radius_sum(distance_squared, left_radius, right_radius)
}

pub(super) fn distance_squared_lte_radius_sum(
    distance_squared: f64,
    left_radius: Real,
    right_radius: Real,
) -> bool {
    let radius_sum = f64::from(left_radius) + f64::from(right_radius);
    let radius_squared = radius_sum * radius_sum;
    distance_squared.is_finite() && radius_squared.is_finite() && distance_squared <= radius_squared
}

pub(super) fn closest_point_on_capsule_segment_y(
    point: Vec3,
    capsule: CapsuleOverlapProxy,
) -> Vec3 {
    let min_y = capsule.center.y - capsule.half_height;
    let max_y = capsule.center.y + capsule.half_height;
    Vec3::new(
        capsule.center.x,
        point.y.clamp(min_y, max_y),
        capsule.center.z,
    )
}

pub(super) fn segment_segment_distance_squared_y(
    left: CapsuleOverlapProxy,
    right: CapsuleOverlapProxy,
) -> f64 {
    let left_min = f64::from(left.center.y) - f64::from(left.half_height);
    let left_max = f64::from(left.center.y) + f64::from(left.half_height);
    let right_min = f64::from(right.center.y) - f64::from(right.half_height);
    let right_max = f64::from(right.center.y) + f64::from(right.half_height);
    let y_gap = interval_interval_gap(left_min, left_max, right_min, right_max);
    let x_gap = f64::from(left.center.x) - f64::from(right.center.x);
    let z_gap = f64::from(left.center.z) - f64::from(right.center.z);
    x_gap * x_gap + y_gap * y_gap + z_gap * z_gap
}

pub(super) fn capsule_segment_aabb_distance_squared_y(
    capsule: CapsuleOverlapProxy,
    box_proxy: BoxOverlapProxy,
) -> f64 {
    let segment_min_y = f64::from(capsule.center.y) - f64::from(capsule.half_height);
    let segment_max_y = f64::from(capsule.center.y) + f64::from(capsule.half_height);
    let x_gap = point_interval_gap(
        f64::from(capsule.center.x),
        f64::from(box_proxy.min.x),
        f64::from(box_proxy.max.x),
    );
    let y_gap = interval_interval_gap(
        segment_min_y,
        segment_max_y,
        f64::from(box_proxy.min.y),
        f64::from(box_proxy.max.y),
    );
    let z_gap = point_interval_gap(
        f64::from(capsule.center.z),
        f64::from(box_proxy.min.z),
        f64::from(box_proxy.max.z),
    );
    x_gap * x_gap + y_gap * y_gap + z_gap * z_gap
}
