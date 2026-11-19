use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsQueryFilter, PhysicsShapeCastHit,
    PhysicsShapeCastQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};

use super::filter::PreparedPhysicsQueryFilter;
use super::geometry::{
    box_geometry_is_valid, collider_aabb, collider_geometry_is_valid, finite_aabb_bounds,
    scaled_box_half_extents,
};
use super::mode::collect_query_mode;
use super::overlap::colliders_overlap;
use super::raycast::ray_cast_collider;

pub(super) fn shape_cast_query(
    sync: &PhysicsWorldSyncState,
    query: &PhysicsShapeCastQuery,
    filter: &PhysicsQueryFilter,
) -> Vec<PhysicsShapeCastHit> {
    let query_collider = PhysicsColliderSyncState {
        entity: 0,
        shape: query.shape.clone(),
        sensor: false,
        layer: 0,
        collision_group: 0,
        collision_mask: u32::MAX,
        material: None,
        material_override: None,
        transform: query.origin_transform,
    };
    if !collider_geometry_is_valid(&query_collider) {
        return Vec::new();
    }

    let direction = Vec3::from_array(query.direction).normalize_or_zero();
    if !direction.is_finite()
        || direction.length_squared() <= Real::EPSILON
        || !query.max_distance.is_finite()
        || query.max_distance < 0.0
    {
        return Vec::new();
    }
    let Some((query_min, query_max)) = collider_aabb(&query_collider) else {
        return Vec::new();
    };
    let query_center = (query_min + query_max) * 0.5;
    let query_half_extents = (query_max - query_min) * 0.5;
    // Zero-length casts keep the old overlap and expanded-proxy behavior.
    let moving_query_half_extents = if query.max_distance > 0.0 {
        sweep_box_bounds(&query_collider, (query_min, query_max))
            .map(|(min, max)| (max - min) * 0.5)
            .filter(|half| half.is_finite())
    } else {
        None
    };

    let prepared_filter = PreparedPhysicsQueryFilter::new(filter);
    collect_query_mode(
        sync.colliders
            .iter()
            .filter(|collider| prepared_filter.matches(collider))
            .filter_map(|collider| {
                if colliders_overlap(&query_collider, collider) {
                    return Some(PhysicsShapeCastHit {
                        entity: collider.entity,
                        distance: 0.0,
                        position: query_center.to_array(),
                        normal: [0.0; 3],
                    });
                }

                let (expanded_center, expanded_half_extents) =
                    expanded_sweep_box(collider, query_half_extents, moving_query_half_extents)?;
                let expanded = PhysicsColliderSyncState {
                    entity: collider.entity,
                    shape: PhysicsColliderShape::Box {
                        half_extents: expanded_half_extents.to_array(),
                    },
                    sensor: collider.sensor,
                    layer: collider.layer,
                    collision_group: collider.collision_group,
                    collision_mask: collider.collision_mask,
                    material: collider.material.clone(),
                    material_override: collider.material_override.clone(),
                    transform: Transform::from_translation(expanded_center),
                };
                ray_cast_collider(query_center, direction, query.max_distance, &expanded).map(
                    |hit| PhysicsShapeCastHit {
                        entity: hit.entity,
                        distance: hit.distance,
                        position: hit.position,
                        normal: hit.normal,
                    },
                )
            }),
        query.mode,
        |left, right| {
            left.distance
                .total_cmp(&right.distance)
                .then(left.entity.cmp(&right.entity))
        },
    )
}

fn expanded_sweep_box(
    collider: &PhysicsColliderSyncState,
    query_half_extents: Vec3,
    moving_query_half_extents: Option<Vec3>,
) -> Option<(Vec3, Vec3)> {
    let legacy_target = collider_aabb(collider)?;
    let preferred = moving_query_half_extents.and_then(|query_half| {
        let target = sweep_box_bounds(collider, legacy_target)?;
        if target == legacy_target && query_half == query_half_extents {
            return None;
        }
        let (center, half) = expanded_box_geometry(target, query_half);
        finite_aabb_bounds(center, half)?;
        Some((center, half))
    });
    // Even finite individual bounds can overflow when expanded. Keep the old
    // pair in that case; do not retry a geometric miss with the old bounds.
    Some(preferred.unwrap_or_else(|| expanded_box_geometry(legacy_target, query_half_extents)))
}

fn expanded_box_geometry((target_min, target_max): (Vec3, Vec3), query_half: Vec3) -> (Vec3, Vec3) {
    let expanded_min = target_min - query_half;
    let expanded_max = target_max + query_half;
    (
        (expanded_min + expanded_max) * 0.5,
        (expanded_max - expanded_min) * 0.5,
    )
}

fn sweep_box_bounds(
    collider: &PhysicsColliderSyncState,
    legacy_bounds: (Vec3, Vec3),
) -> Option<(Vec3, Vec3)> {
    let PhysicsColliderShape::Box { half_extents } = collider.shape else {
        return Some(legacy_bounds);
    };
    let rotation = collider.transform.rotation;
    // Quat multiplication requires a unit rotation; other poses retain the old bounds.
    if (rotation.x == 0.0 && rotation.y == 0.0 && rotation.z == 0.0)
        || !rotation.is_normalized()
        || !box_geometry_is_valid(half_extents)
    {
        return Some(legacy_bounds);
    }

    let half = scaled_box_half_extents(half_extents, collider.transform.scale)?;
    let world_half = (rotation * Vec3::X * half.x).abs()
        + (rotation * Vec3::Y * half.y).abs()
        + (rotation * Vec3::Z * half.z).abs();
    finite_aabb_bounds(collider.transform.translation, world_half)
}
