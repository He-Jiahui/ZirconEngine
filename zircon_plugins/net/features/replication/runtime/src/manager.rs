use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncComponentDescriptor, SyncDelta, SyncFieldValue,
    SyncInterestDescriptor, SyncObjectSnapshot, SyncReplicationBudget,
    SyncReplicationScheduleReport,
};

const MILLIS_PER_SECOND: u64 = 1_000;

#[derive(Clone, Debug, Default)]
pub struct NetReplicationRuntimeManager {
    state: Arc<Mutex<NetReplicationRuntimeState>>,
}

#[derive(Debug, Default)]
struct NetReplicationRuntimeState {
    descriptors: HashMap<String, SyncComponentDescriptor>,
    snapshots: HashMap<(NetObjectId, String), SyncObjectSnapshot>,
    sequences: HashMap<(NetObjectId, String), u64>,
    interests: HashMap<NetSessionId, SyncInterestDescriptor>,
    last_replication_ms: HashMap<(NetSessionId, NetObjectId, String), u64>,
}

impl NetReplicationRuntimeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component(&self, descriptor: SyncComponentDescriptor) {
        self.state
            .lock()
            .expect("net replication state mutex poisoned")
            .descriptors
            .insert(descriptor.component_type.clone(), descriptor);
    }

    pub fn set_interest(&self, interest: SyncInterestDescriptor) {
        self.state
            .lock()
            .expect("net replication state mutex poisoned")
            .interests
            .insert(interest.session, interest);
    }

    pub fn publish_snapshot(
        &self,
        object: NetObjectId,
        component_type: &str,
        fields: impl IntoIterator<Item = SyncFieldValue>,
    ) -> Option<SyncDelta> {
        let mut state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let descriptor = state.descriptors.get(component_type)?.clone();
        let fields = fields.into_iter().collect::<Vec<_>>();
        let key = (object, component_type.to_string());
        let previous = state.snapshots.get(&key);
        let changed_fields = fields
            .iter()
            .filter(|field| {
                previous.map_or(true, |snapshot| {
                    snapshot
                        .fields
                        .iter()
                        .find(|old| old.name == field.name)
                        .map_or(true, |old| old.bytes != field.bytes)
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        let sequence = {
            let sequence = state.sequences.entry(key.clone()).or_insert(0);
            *sequence += 1;
            *sequence
        };
        let delta = SyncDelta::new(object, component_type, sequence, changed_fields);
        state
            .snapshots
            .insert(key, SyncObjectSnapshot::new(object, &descriptor, fields));
        Some(delta)
    }

    pub fn visible_snapshots(&self, session: NetSessionId) -> Vec<SyncObjectSnapshot> {
        let state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let interest = state.interests.get(&session);
        state
            .snapshots
            .values()
            .filter(|snapshot| {
                interest.map_or(true, |interest| {
                    interest.allows_group(snapshot.interest_group.as_deref())
                })
            })
            .cloned()
            .collect()
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
                        .unwrap_or(MILLIS_PER_SECOND),
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

    pub fn despawn_object(&self, object: NetObjectId) -> Vec<SyncObjectSnapshot> {
        let mut state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let removed_keys = state
            .snapshots
            .keys()
            .filter(|(snapshot_object, _)| *snapshot_object == object)
            .cloned()
            .collect::<Vec<_>>();
        removed_keys
            .into_iter()
            .filter_map(|key| {
                state.sequences.remove(&key);
                state.remove_replication_times(object, &key.1);
                state.snapshots.remove(&key)
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
struct ScheduledSnapshotCandidate {
    snapshot: SyncObjectSnapshot,
    priority: u16,
    update_interval_ms: u64,
}

impl NetReplicationRuntimeState {
    fn allows_interest(&self, session: NetSessionId, snapshot: &SyncObjectSnapshot) -> bool {
        self.interests.get(&session).map_or(true, |interest| {
            interest.allows_group(snapshot.interest_group.as_deref())
        })
    }

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

    fn remove_replication_times(&mut self, object: NetObjectId, component_type: &str) {
        self.last_replication_ms
            .retain(|(_, replicated_object, replicated_component), _| {
                *replicated_object != object || replicated_component != component_type
            });
    }
}

fn update_interval_ms(descriptor: &SyncComponentDescriptor) -> u64 {
    if descriptor.update_hz == 0 {
        return MILLIS_PER_SECOND;
    }
    MILLIS_PER_SECOND.div_ceil(u64::from(descriptor.update_hz))
}

fn snapshot_payload_bytes(snapshot: &SyncObjectSnapshot) -> usize {
    snapshot.fields.iter().map(|field| field.bytes.len()).sum()
}

fn replication_time_key(
    session: NetSessionId,
    snapshot: &SyncObjectSnapshot,
) -> (NetSessionId, NetObjectId, String) {
    (session, snapshot.object, snapshot.component_type.clone())
}

pub fn net_replication_runtime_manager() -> NetReplicationRuntimeManager {
    NetReplicationRuntimeManager::new()
}
