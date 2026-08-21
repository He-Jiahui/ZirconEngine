use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsQueryFilter, PhysicsShapeOverlapHit,
    PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::math::Transform;

use super::super::filter::PreparedPhysicsQueryFilter;
use super::super::geometry::collider_geometry_is_valid;
use super::super::mode::collect_query_mode;
use super::pairwise::colliders_overlap;

pub(super) fn shape_overlap_query(
    sync: &PhysicsWorldSyncState,
    query: &PhysicsShapeOverlapQuery,
    filter: &PhysicsQueryFilter,
) -> Vec<PhysicsShapeOverlapHit> {
    let Some(query_collider) = shape_query_collider(&query.shape, query.transform) else {
        return Vec::new();
    };

    let prepared_filter = PreparedPhysicsQueryFilter::new(filter);
    let origin = query.transform.translation;
    collect_query_mode(
        sync.colliders
            .iter()
            .filter(|collider| prepared_filter.matches(collider))
            .filter(|collider| colliders_overlap(&query_collider, collider))
            .map(|collider| PhysicsShapeOverlapHit {
                entity: collider.entity,
                shape: collider.shape.clone(),
                transform: collider.transform,
                sensor: collider.sensor,
                layer: collider.layer,
                collision_group: collider.collision_group,
            }),
        query.mode,
        |left, right| {
            left.transform
                .translation
                .distance_squared(origin)
                .total_cmp(&right.transform.translation.distance_squared(origin))
                .then(left.entity.cmp(&right.entity))
        },
    )
}

fn shape_query_collider(
    shape: &PhysicsColliderShape,
    transform: Transform,
) -> Option<PhysicsColliderSyncState> {
    let collider = PhysicsColliderSyncState {
        entity: 0,
        shape: shape.clone(),
        sensor: false,
        layer: 0,
        collision_group: 0,
        collision_mask: u32::MAX,
        material: None,
        material_override: None,
        transform,
    };
    collider_geometry_is_valid(&collider).then_some(collider)
}
