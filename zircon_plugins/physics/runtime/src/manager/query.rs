use std::sync::Arc;

use zircon_runtime::core::framework::physics::{
    PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

use crate::backend::builtin::{
    collect_query_mode, ray_cast_collider, shape_cast_query, shape_overlap_query,
    PreparedPhysicsQueryFilter,
};

use super::validation::{array3_is_finite, normalized_ray_direction, transform_is_finite};
use super::DefaultPhysicsManager;

pub(super) fn ray_cast(
    manager: &DefaultPhysicsManager,
    query: &PhysicsRayCastQuery,
) -> Vec<PhysicsRayCastHit> {
    if !query.max_distance.is_finite()
        || !array3_is_finite(query.origin)
        || !array3_is_finite(query.direction)
    {
        return Vec::new();
    }
    let Some(direction) = normalized_ray_direction(query.direction) else {
        return Vec::new();
    };
    if query.max_distance <= 0.0 {
        return Vec::new();
    }
    let Some(sync) = synchronized_world(manager, query.world) else {
        return Vec::new();
    };

    let filter = PreparedPhysicsQueryFilter::new(&query.filter);
    collect_query_mode(
        sync.colliders
            .iter()
            .filter(|collider| filter.matches(collider))
            .filter_map(|collider| {
                ray_cast_collider(
                    Vec3::from_array(query.origin),
                    direction,
                    query.max_distance,
                    collider,
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

pub(super) fn shape_overlap(
    manager: &DefaultPhysicsManager,
    query: &PhysicsShapeOverlapQuery,
) -> Vec<PhysicsShapeOverlapHit> {
    synchronized_world(manager, query.world)
        .map(|sync| shape_overlap_query(&sync, query, &query.filter))
        .unwrap_or_default()
}

pub(super) fn shape_cast(
    manager: &DefaultPhysicsManager,
    query: &PhysicsShapeCastQuery,
) -> Vec<PhysicsShapeCastHit> {
    if !query.max_distance.is_finite()
        || query.max_distance < 0.0
        || !array3_is_finite(query.direction)
        || !transform_is_finite(query.origin_transform)
    {
        return Vec::new();
    }

    synchronized_world(manager, query.world)
        .map(|sync| shape_cast_query(&sync, query, &query.filter))
        .unwrap_or_default()
}

pub(super) fn synchronized_world(
    manager: &DefaultPhysicsManager,
    world: WorldHandle,
) -> Option<Arc<PhysicsWorldSyncState>> {
    manager.synchronized_world_snapshot(world)
}
