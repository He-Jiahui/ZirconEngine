use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncComponentDescriptor, SyncDelta, SyncFieldValue,
    SyncInterestDescriptor, SyncObjectSnapshot,
};

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
}

pub fn net_replication_runtime_manager() -> NetReplicationRuntimeManager {
    NetReplicationRuntimeManager::new()
}
