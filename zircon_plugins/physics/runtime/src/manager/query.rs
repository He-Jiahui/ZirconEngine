use zircon_runtime::core::framework::physics::{
    PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

use crate::backend::builtin::{collider_matches_query, ray_cast_collider, shape_overlap_query};

use super::poison_recovery::recover_lock;
use super::validation::{array3_is_finite, normalized_ray_direction, transform_is_finite};
use super::DefaultPhysicsManager;

pub(super) fn ray_cast(
    manager: &DefaultPhysicsManager,
    query: &PhysicsRayCastQuery,
) -> Option<PhysicsRayCastHit> {
    if !query.max_distance.is_finite()
        || !array3_is_finite(query.origin)
        || !array3_is_finite(query.direction)
    {
        return None;
    }
    let Some(direction) = normalized_ray_direction(query.direction) else {
        return None;
    };
    if query.max_distance <= 0.0 {
        return None;
    }

    synchronized_world(manager, query.world)?
        .colliders
        .iter()
        .filter(|collider| collider_matches_query(query, collider))
        .filter_map(|collider| {
            ray_cast_collider(
                Vec3::from_array(query.origin),
                direction,
                query.max_distance,
                collider,
            )
        })
        .min_by(|left, right| {
            left.distance
                .partial_cmp(&right.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(super) fn shape_overlap(
    manager: &DefaultPhysicsManager,
    query: &PhysicsShapeOverlapQuery,
) -> Vec<PhysicsShapeOverlapHit> {
    synchronized_world(manager, query.world)
        .map(|sync| shape_overlap_query(&sync, query))
        .unwrap_or_default()
}

pub(super) fn shape_cast(
    manager: &DefaultPhysicsManager,
    query: &PhysicsShapeCastQuery,
) -> Option<PhysicsShapeCastHit> {
    if !query.max_distance.is_finite()
        || query.max_distance < 0.0
        || !array3_is_finite(query.direction)
        || !transform_is_finite(query.origin_transform)
    {
        return None;
    }

    shape_overlap(
        manager,
        &PhysicsShapeOverlapQuery {
            world: query.world,
            shape: query.shape.clone(),
            transform: query.origin_transform,
            filter: query.filter.clone(),
        },
    )
    .into_iter()
    .next()
    .map(|hit| PhysicsShapeCastHit {
        entity: hit.entity,
        distance: 0.0,
        position: query.origin_transform.translation.to_array(),
        normal: [0.0, 0.0, 0.0],
    })
}

fn synchronized_world(
    manager: &DefaultPhysicsManager,
    world: WorldHandle,
) -> Option<PhysicsWorldSyncState> {
    recover_lock(&manager.synced_worlds).get(&world).cloned()
}
