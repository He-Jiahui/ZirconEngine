mod distance;
mod pairwise;
mod proxies;
mod query;

use zircon_runtime::core::framework::physics::{
    PhysicsColliderSyncState, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery,
    PhysicsWorldSyncState,
};

pub(super) fn colliders_overlap(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
) -> bool {
    pairwise::colliders_overlap(left, right)
}

pub(super) fn shape_overlap_query(
    sync: &PhysicsWorldSyncState,
    query: &PhysicsShapeOverlapQuery,
) -> Vec<PhysicsShapeOverlapHit> {
    query::shape_overlap_query(sync, query)
}
