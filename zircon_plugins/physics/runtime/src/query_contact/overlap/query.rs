use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsColliderSyncState, PhysicsShapeOverlapHit,
    PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::math::Transform;

use super::super::filter::collider_matches_filter;
use super::super::geometry::collider_geometry_is_valid;
use super::pairwise::colliders_overlap;

pub(super) fn shape_overlap_query(
    sync: &PhysicsWorldSyncState,
    query: &PhysicsShapeOverlapQuery,
) -> Vec<PhysicsShapeOverlapHit> {
    let Some(query_collider) = shape_query_collider(&query.shape, query.transform) else {
        return Vec::new();
    };

    sync.colliders
        .iter()
        .filter(|collider| collider_matches_filter(&query.filter, collider))
        .filter(|collider| colliders_overlap(&query_collider, collider))
        .map(|collider| PhysicsShapeOverlapHit {
            entity: collider.entity,
            shape: collider.shape.clone(),
            transform: collider.transform,
            sensor: collider.sensor,
            layer: collider.layer,
            collision_group: collider.collision_group,
        })
        .collect()
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
