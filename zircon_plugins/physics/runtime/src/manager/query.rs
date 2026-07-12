use zircon_runtime::core::framework::physics::{
    PhysicsQueryMode, PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsShapeCastHit,
    PhysicsShapeCastQuery, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Real, Vec3};

use crate::backend::builtin::{
    collider_matches_query, ray_cast_collider, shape_cast_query, shape_overlap_query,
};

use super::poison_recovery::recover_lock;
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

    let mut hits = sync
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
        .collect::<Vec<_>>();
    apply_distance_mode(&mut hits, query.mode, |hit| (hit.distance, hit.entity));
    hits
}

pub(super) fn shape_overlap(
    manager: &DefaultPhysicsManager,
    query: &PhysicsShapeOverlapQuery,
) -> Vec<PhysicsShapeOverlapHit> {
    let mut hits = synchronized_world(manager, query.world)
        .map(|sync| shape_overlap_query(&sync, query))
        .unwrap_or_default();
    apply_overlap_mode(&mut hits, query);
    hits
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

    let mut hits = synchronized_world(manager, query.world)
        .map(|sync| shape_cast_query(&sync, query))
        .unwrap_or_default();
    apply_distance_mode(&mut hits, query.mode, |hit| (hit.distance, hit.entity));
    hits
}

fn apply_distance_mode<T>(
    hits: &mut Vec<T>,
    mode: PhysicsQueryMode,
    key: impl Fn(&T) -> (Real, EntityId),
) {
    match mode {
        PhysicsQueryMode::First => hits.truncate(1),
        PhysicsQueryMode::Closest => {
            if let Some(index) = hits
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| compare_distance_key(key(left), key(right)))
                .map(|(index, _)| index)
            {
                hits.swap(0, index);
                hits.truncate(1);
            }
        }
        PhysicsQueryMode::All => {
            hits.sort_by(|left, right| compare_distance_key(key(left), key(right)));
        }
    }
}

fn compare_distance_key(left: (Real, EntityId), right: (Real, EntityId)) -> std::cmp::Ordering {
    left.0.total_cmp(&right.0).then(left.1.cmp(&right.1))
}

fn apply_overlap_mode(hits: &mut Vec<PhysicsShapeOverlapHit>, query: &PhysicsShapeOverlapQuery) {
    let origin = query.transform.translation;
    match query.mode {
        PhysicsQueryMode::First => hits.truncate(1),
        PhysicsQueryMode::Closest => {
            hits.sort_by(|left, right| {
                left.transform
                    .translation
                    .distance_squared(origin)
                    .total_cmp(&right.transform.translation.distance_squared(origin))
                    .then(left.entity.cmp(&right.entity))
            });
            hits.truncate(1);
        }
        PhysicsQueryMode::All => hits.sort_by(|left, right| {
            left.transform
                .translation
                .distance_squared(origin)
                .total_cmp(&right.transform.translation.distance_squared(origin))
                .then(left.entity.cmp(&right.entity))
        }),
    }
}

fn synchronized_world(
    manager: &DefaultPhysicsManager,
    world: WorldHandle,
) -> Option<PhysicsWorldSyncState> {
    recover_lock(&manager.synced_worlds).get(&world).cloned()
}
