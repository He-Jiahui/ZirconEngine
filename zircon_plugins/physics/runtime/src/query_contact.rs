mod contact;
mod filter;
mod geometry;
mod overlap;
mod raycast;

use zircon_runtime::core::framework::physics::{
    PhysicsColliderSyncState, PhysicsContactEvent, PhysicsRayCastHit, PhysicsRayCastQuery,
    PhysicsSettings, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::math::{Real, Vec3};

pub(super) fn compute_contact_events(
    sync: &PhysicsWorldSyncState,
    settings: &PhysicsSettings,
) -> Vec<PhysicsContactEvent> {
    contact::compute_contact_events(sync, settings)
}

pub(super) fn collider_matches_query(
    query: &PhysicsRayCastQuery,
    collider: &PhysicsColliderSyncState,
) -> bool {
    filter::collider_matches_query(query, collider)
}

pub(super) fn colliders_can_interact(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
    settings: &PhysicsSettings,
) -> bool {
    filter::colliders_can_interact(left, right, settings)
}

pub(super) fn colliders_overlap(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
) -> bool {
    overlap::colliders_overlap(left, right)
}

pub(super) fn shape_overlap_query(
    sync: &PhysicsWorldSyncState,
    query: &PhysicsShapeOverlapQuery,
) -> Vec<PhysicsShapeOverlapHit> {
    overlap::shape_overlap_query(sync, query)
}

pub(super) fn ray_cast_collider(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    collider: &PhysicsColliderSyncState,
) -> Option<PhysicsRayCastHit> {
    raycast::ray_cast_collider(origin, direction, max_distance, collider)
}
