//! 按声明优先级、对象/组件顺序和更新间隔，为单个 session 在数量/字节预算下选择快照。
//! 报告只表示 manager 已选中，未实际发送；上层需把投递结果与下次调度状态协调。

use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncObjectSnapshot, SyncReplicationBudget,
    SyncReplicationScheduleReport,
};

use super::NetReplicationRuntimeManager;
use super::budget::update_interval_ms;
use super::snapshot::snapshot_payload_bytes;

#[derive(Clone, Debug)]
struct ScheduledSnapshotCandidate {
    key: (NetObjectId, String),
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
        let candidates = ordered_snapshot_candidates(&state);

        for candidate in candidates {
            let Some(snapshot) = state.snapshots.get(&candidate.key) else {
                continue;
            };
            let snapshot = if !state.allows_interest(session, snapshot) {
                report.skipped_by_interest += 1;
                continue;
            } else if !state.snapshot_due(
                session,
                snapshot,
                tick_time_ms,
                candidate.update_interval_ms,
            ) {
                report.skipped_not_due += 1;
                continue;
            } else {
                let snapshot_bytes = snapshot_payload_bytes(snapshot);
                if !budget.allows_snapshot_count(report.sent_snapshots.len())
                    || !budget.allows_byte_count(report.used_bytes, snapshot_bytes)
                {
                    report.deferred_snapshots += 1;
                    continue;
                }
                report.used_bytes += snapshot_bytes;
                snapshot.clone()
            };

            // TODO: [CR-PLUGIN-NET-0024] 选中即计为已复制，尚无发送确认；下游失败时需要说明回滚/重试与更新频率合同。
            state.mark_snapshot_replicated(session, &snapshot, tick_time_ms);
            report.sent_snapshots.push(snapshot);
        }
        report
    }
}

fn ordered_snapshot_candidates(
    state: &super::state::NetReplicationRuntimeState,
) -> Vec<ScheduledSnapshotCandidate> {
    let mut candidates = state
        .snapshots
        .iter()
        .map(|(key, snapshot)| {
            let descriptor = state.descriptors.get(&snapshot.component_type);
            ScheduledSnapshotCandidate {
                key: key.clone(),
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
            .then_with(|| left.key.0.raw().cmp(&right.key.0.raw()))
            .then_with(|| left.key.1.cmp(&right.key.1))
    });
    candidates
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

#[cfg(test)]
#[path = "tests/schedule_payload_clone_tests.rs"]
mod payload_clone_tests;
