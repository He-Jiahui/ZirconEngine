use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncObjectSnapshot, SyncReplicationBudget,
    SyncReplicationScheduleReport,
};

use super::budget::update_interval_ms;
use super::snapshot::snapshot_payload_bytes;
use super::NetReplicationRuntimeManager;

#[derive(Clone, Debug)]
struct ScheduledSnapshotCandidate {
    snapshot: SyncObjectSnapshot,
    priority: u16,
    update_interval_ms: u64,
}

impl NetReplicationRuntimeManager {
    pub(in crate::manager) fn scheduled_snapshots_impl(
        &self,
        session: NetSessionId,
        tick_time_ms: u64,
        budget: SyncReplicationBudget,
    ) -> SyncReplicationScheduleReport {
        let mut state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let mut report = SyncReplicationScheduleReport::new(session, tick_time_ms, budget);
        let mut candidates = state
            .snapshots
            .values()
            .map(|snapshot| {
                let descriptor = state.descriptors.get(&snapshot.component_type);
                ScheduledSnapshotCandidate {
                    snapshot: snapshot.clone(),
                    priority: descriptor
                        .map(|descriptor| descriptor.replication_priority)
                        .unwrap_or_default(),
                    update_interval_ms: descriptor
                        .map(update_interval_ms)
                        .unwrap_or(super::MILLIS_PER_SECOND),
                }
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.snapshot.object.raw().cmp(&right.snapshot.object.raw()))
                .then_with(|| {
                    left.snapshot
                        .component_type
                        .cmp(&right.snapshot.component_type)
                })
        });

        for candidate in candidates {
            let snapshot = candidate.snapshot;
            if !state.allows_interest(session, &snapshot) {
                report.skipped_by_interest += 1;
                continue;
            }
            if !state.snapshot_due(
                session,
                &snapshot,
                tick_time_ms,
                candidate.update_interval_ms,
            ) {
                report.skipped_not_due += 1;
                continue;
            }

            let snapshot_bytes = snapshot_payload_bytes(&snapshot);
            if !budget.allows_snapshot_count(report.sent_snapshots.len())
                || !budget.allows_byte_count(report.used_bytes, snapshot_bytes)
            {
                report.deferred_snapshots += 1;
                continue;
            }

            state.mark_snapshot_replicated(session, &snapshot, tick_time_ms);
            report.used_bytes += snapshot_bytes;
            report.sent_snapshots.push(snapshot);
        }
        report
    }
}

impl super::state::NetReplicationRuntimeState {
    fn snapshot_due(
        &self,
        session: NetSessionId,
        snapshot: &SyncObjectSnapshot,
        tick_time_ms: u64,
        update_interval_ms: u64,
    ) -> bool {
        let key = replication_time_key(session, snapshot);
        self.last_replication_ms
            .get(&key)
            .is_none_or(|last_time_ms| {
                tick_time_ms.saturating_sub(*last_time_ms) >= update_interval_ms
            })
    }

    fn mark_snapshot_replicated(
        &mut self,
        session: NetSessionId,
        snapshot: &SyncObjectSnapshot,
        tick_time_ms: u64,
    ) {
        self.last_replication_ms
            .insert(replication_time_key(session, snapshot), tick_time_ms);
    }
}

pub(in crate::manager) fn replication_time_key(
    session: NetSessionId,
    snapshot: &SyncObjectSnapshot,
) -> (NetSessionId, NetObjectId, String) {
    (session, snapshot.object, snapshot.component_type.clone())
}
