#[path = "query_contact/contact.rs"]
mod contact;
#[path = "query_contact/filter.rs"]
mod filter;
#[path = "query_contact/geometry.rs"]
mod geometry;
#[path = "query_contact/mode.rs"]
mod mode;
#[path = "query_contact/overlap.rs"]
mod overlap;
#[path = "query_contact/raycast.rs"]
mod raycast;
#[path = "query_contact/sweep.rs"]
mod sweep;

use zircon_runtime::core::framework::physics::{
    PhysicsColliderSyncState, PhysicsContactEvent, PhysicsQueryFilter, PhysicsRayCastHit,
    PhysicsSettings, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsWorldSyncState,
};
use zircon_runtime::core::math::{Real, Vec3};

pub(crate) fn compute_contact_events(
    sync: &PhysicsWorldSyncState,
    settings: &PhysicsSettings,
) -> Vec<PhysicsContactEvent> {
    contact::compute_contact_events(sync, settings)
}

pub(crate) use filter::PreparedPhysicsQueryFilter;
pub(crate) use mode::{append_query_mode, collect_query_mode};

pub(crate) fn colliders_can_interact(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
    settings: &PhysicsSettings,
) -> bool {
    filter::colliders_can_interact(left, right, settings)
}

pub(crate) fn colliders_overlap(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
) -> bool {
    overlap::colliders_overlap(left, right)
}

pub(crate) fn shape_overlap_query(
    sync: &PhysicsWorldSyncState,
    query: &PhysicsShapeOverlapQuery,
    filter: &PhysicsQueryFilter,
) -> Vec<PhysicsShapeOverlapHit> {
    overlap::shape_overlap_query(sync, query, filter)
}

pub(crate) fn ray_cast_collider(
    origin: Vec3,
    direction: Vec3,
    max_distance: Real,
    collider: &PhysicsColliderSyncState,
) -> Option<PhysicsRayCastHit> {
    raycast::ray_cast_collider(origin, direction, max_distance, collider)
}

pub(crate) fn shape_cast_query(
    sync: &PhysicsWorldSyncState,
    query: &zircon_runtime::core::framework::physics::PhysicsShapeCastQuery,
    filter: &PhysicsQueryFilter,
) -> Vec<zircon_runtime::core::framework::physics::PhysicsShapeCastHit> {
    sweep::shape_cast_query(sync, query, filter)
}
