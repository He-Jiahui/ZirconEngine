use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldQuery, WorldQueryResult,
};

use super::RuntimeDynamicSession;

impl RuntimeDynamicSession {
    /// Queries the session-owned runtime world through the transport-neutral DTO contract.
    pub(super) fn query_world(&self, query: WorldQuery) -> WorldQueryResult {
        self.level.with_world(|world| world.query_world(&query))
    }

    /// Registers one revocable session-local world watch.
    pub(super) fn watch_world(&self, registration: WatchRegistration) -> WatchToken {
        self.level.watch_world(registration)
    }

    /// Revokes one session-local watch and reports whether it was still live.
    pub(super) fn unwatch_world(&self, token: WatchToken) -> bool {
        self.level.unwatch_world(token)
    }

    /// Seals every runtime fact observed since the previous serialized drain.
    pub(super) fn drain_world_invalidations(&self) -> Vec<InvalidationBatch> {
        self.level.drain_world_invalidations()
    }
}
