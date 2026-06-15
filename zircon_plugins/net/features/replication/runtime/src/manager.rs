use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncComponentDescriptor, SyncDelta, SyncFieldValue,
    SyncInterestDescriptor, SyncObjectSnapshot, SyncReplicationBudget,
    SyncReplicationScheduleReport,
};

mod apply;
mod budget;
mod collect;
mod interest;
mod lifecycle;
mod registry;
mod schedule;
mod snapshot;
mod state;
mod table;

pub use table::{NetReplicationTable, NetReplicationTableEntry};

pub(in crate::manager) const MILLIS_PER_SECOND: u64 = 1_000;

#[derive(Clone, Debug, Default)]
pub struct NetReplicationRuntimeManager {
    pub(in crate::manager) state: Arc<Mutex<state::NetReplicationRuntimeState>>,
}

impl NetReplicationRuntimeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component(&self, descriptor: SyncComponentDescriptor) {
        self.register_component_impl(descriptor);
    }

    pub fn set_interest(&self, interest: SyncInterestDescriptor) {
        self.set_interest_impl(interest);
    }

    pub fn publish_snapshot(
        &self,
        object: NetObjectId,
        component_type: &str,
        fields: impl IntoIterator<Item = SyncFieldValue>,
    ) -> Option<SyncDelta> {
        self.collect_snapshot_delta(object, component_type, fields)
    }

    pub fn visible_snapshots(&self, session: NetSessionId) -> Vec<SyncObjectSnapshot> {
        self.visible_snapshots_impl(session)
    }

    pub fn late_join_snapshots(&self, session: NetSessionId) -> Vec<SyncObjectSnapshot> {
        self.visible_snapshots(session)
    }

    pub fn scheduled_snapshots(
        &self,
        session: NetSessionId,
        tick_time_ms: u64,
        budget: SyncReplicationBudget,
    ) -> SyncReplicationScheduleReport {
        self.scheduled_snapshots_impl(session, tick_time_ms, budget)
    }

    pub fn despawn_object(&self, object: NetObjectId) -> Vec<SyncObjectSnapshot> {
        self.despawn_object_impl(object)
    }
}

pub fn net_replication_runtime_manager() -> NetReplicationRuntimeManager {
    NetReplicationRuntimeManager::new()
}
