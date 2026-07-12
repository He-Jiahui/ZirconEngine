use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsWorldSyncState,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};

use super::filter::collider_matches_filter;
use super::geometry::{collider_aabb, collider_geometry_is_valid};
use super::overlap::colliders_overlap;
use super::raycast::ray_cast_collider;

pub(super) fn shape_cast_query(
    sync: &PhysicsWorldSyncState,
    query: &PhysicsShapeCastQuery,
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

    sync.colliders
        .iter()
        .filter(|collider| collider_matches_filter(&query.filter, collider))
        .filter_map(|collider| {
            if colliders_overlap(&query_collider, collider) {
                return Some(PhysicsShapeCastHit {
                    entity: collider.entity,
                    distance: 0.0,
                    position: query_center.to_array(),
                    normal: [0.0; 3],
                });
            }

            let (target_min, target_max) = collider_aabb(collider)?;
            let expanded_min = target_min - query_half_extents;
            let expanded_max = target_max + query_half_extents;
            let expanded_center = (expanded_min + expanded_max) * 0.5;
            let expanded_half_extents = (expanded_max - expanded_min) * 0.5;
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
            ray_cast_collider(query_center, direction, query.max_distance, &expanded).map(|hit| {
                PhysicsShapeCastHit {
                    entity: hit.entity,
                    distance: hit.distance,
                    position: hit.position,
                    normal: hit.normal,
                }
            })
        })
        .collect()
}
