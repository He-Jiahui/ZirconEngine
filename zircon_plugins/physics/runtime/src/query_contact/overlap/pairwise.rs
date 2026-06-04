use zircon_runtime::core::framework::physics::{PhysicsColliderShape, PhysicsColliderSyncState};

use super::super::geometry::{collider_aabb, collider_geometry_is_valid};
use super::distance::{
    capsule_segment_aabb_distance_squared_y, closest_point_on_capsule_segment_y,
    distance_squared_lte_radius_sum, point_distance_squared_lte_radius_sum,
    segment_segment_distance_squared_y,
};
use super::proxies::{
    collider_box, collider_capsule_y, collider_sphere, BoxOverlapProxy, CapsuleOverlapProxy,
    SphereOverlapProxy,
};

pub(super) fn colliders_overlap(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
) -> bool {
    if !collider_geometry_is_valid(left) || !collider_geometry_is_valid(right) {
        return false;
    }

    if let (Some(left_sphere), Some(right_sphere)) = (collider_sphere(left), collider_sphere(right))
    {
        return sphere_sphere_overlap(left_sphere, right_sphere);
    }
    if let (Some(sphere), Some(box_proxy)) = (collider_sphere(left), collider_box(right)) {
        return sphere_box_overlap(sphere, box_proxy);
    }
    if let (Some(box_proxy), Some(sphere)) = (collider_box(left), collider_sphere(right)) {
        return sphere_box_overlap(sphere, box_proxy);
    }
    if let (Some(box_proxy), Some(capsule)) = (collider_box(left), collider_capsule_y(right)) {
        return capsule_box_overlap(capsule, box_proxy);
    }
    if let (Some(capsule), Some(box_proxy)) = (collider_capsule_y(left), collider_box(right)) {
        return capsule_box_overlap(capsule, box_proxy);
    }
    if let (Some(sphere), Some(capsule)) = (collider_sphere(left), collider_capsule_y(right)) {
        return sphere_capsule_overlap(sphere, capsule);
    }
    if let (Some(capsule), Some(sphere)) = (collider_capsule_y(left), collider_sphere(right)) {
        return sphere_capsule_overlap(sphere, capsule);
    }
    if let (Some(left_capsule), Some(right_capsule)) =
        (collider_capsule_y(left), collider_capsule_y(right))
    {
        return capsule_capsule_overlap(left_capsule, right_capsule);
    }
    if colliders_are_boxes(left, right) {
        return match (collider_box(left), collider_box(right)) {
            (Some(left_box), Some(right_box)) => box_box_overlap(left_box, right_box),
            _ => false,
        };
    }

    let Some((left_min, left_max)) = collider_aabb(left) else {
        return false;
    };
    let Some((right_min, right_max)) = collider_aabb(right) else {
        return false;
    };
    left_min.x <= right_max.x
        && left_max.x >= right_min.x
        && left_min.y <= right_max.y
        && left_max.y >= right_min.y
        && left_min.z <= right_max.z
        && left_max.z >= right_min.z
}

fn colliders_are_boxes(left: &PhysicsColliderSyncState, right: &PhysicsColliderSyncState) -> bool {
    matches!(left.shape, PhysicsColliderShape::Box { .. })
        && matches!(right.shape, PhysicsColliderShape::Box { .. })
}

fn sphere_sphere_overlap(left: SphereOverlapProxy, right: SphereOverlapProxy) -> bool {
    point_distance_squared_lte_radius_sum(left.center, right.center, left.radius, right.radius)
}

fn sphere_box_overlap(sphere: SphereOverlapProxy, box_proxy: BoxOverlapProxy) -> bool {
    let closest = sphere.center.clamp(box_proxy.min, box_proxy.max);
    point_distance_squared_lte_radius_sum(sphere.center, closest, sphere.radius, 0.0)
}

fn sphere_capsule_overlap(sphere: SphereOverlapProxy, capsule: CapsuleOverlapProxy) -> bool {
    let closest = closest_point_on_capsule_segment_y(sphere.center, capsule);
    point_distance_squared_lte_radius_sum(sphere.center, closest, sphere.radius, capsule.radius)
}

fn capsule_box_overlap(capsule: CapsuleOverlapProxy, box_proxy: BoxOverlapProxy) -> bool {
    distance_squared_lte_radius_sum(
        capsule_segment_aabb_distance_squared_y(capsule, box_proxy),
        capsule.radius,
        0.0,
    )
}

fn capsule_capsule_overlap(left: CapsuleOverlapProxy, right: CapsuleOverlapProxy) -> bool {
    distance_squared_lte_radius_sum(
        segment_segment_distance_squared_y(left, right),
        left.radius,
        right.radius,
    )
}

fn box_box_overlap(left: BoxOverlapProxy, right: BoxOverlapProxy) -> bool {
    left.min.x <= right.max.x
        && left.max.x >= right.min.x
        && left.min.y <= right.max.y
        && left.max.y >= right.min.y
        && left.min.z <= right.max.z
        && left.max.z >= right.min.z
}
