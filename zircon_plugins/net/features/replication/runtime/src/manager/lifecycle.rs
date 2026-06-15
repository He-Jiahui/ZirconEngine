use zircon_runtime::core::framework::net::{NetObjectId, SyncObjectSnapshot};

use super::NetReplicationRuntimeManager;

impl NetReplicationRuntimeManager {
    pub(in crate::manager) fn despawn_object_impl(
        &self,
        object: NetObjectId,
    ) -> Vec<SyncObjectSnapshot> {
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
                state.remove_interpolation_samples(object, &key.1);
                state.snapshots.remove(&key)
            })
            .collect()
    }
}

impl super::state::NetReplicationRuntimeState {
    pub(in crate::manager) fn remove_replication_times(
        &mut self,
        object: NetObjectId,
        component_type: &str,
    ) {
        self.last_replication_ms
            .retain(|(_, replicated_object, replicated_component), _| {
                *replicated_object != object || replicated_component != component_type
            });
    }
}
