use zircon_runtime::core::framework::net::{NetObjectId, SyncDelta, SyncFieldValue};

use super::NetReplicationRuntimeManager;

impl NetReplicationRuntimeManager {
    pub fn collect_snapshot_delta(
        &self,
        object: NetObjectId,
        component_type: &str,
        fields: impl IntoIterator<Item = SyncFieldValue>,
    ) -> Option<SyncDelta> {
        self.publish_snapshot_impl(object, component_type, fields)
    }

    pub fn collect_despawn_deltas(&self, object: NetObjectId) -> Vec<SyncDelta> {
        let mut state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let mut removed_keys = state
            .snapshots
            .keys()
            .filter(|(snapshot_object, _)| *snapshot_object == object)
            .cloned()
            .collect::<Vec<_>>();
        removed_keys.sort_by(|left, right| left.1.cmp(&right.1));

        removed_keys
            .into_iter()
            .filter_map(|key| {
                state.snapshots.remove(&key)?;
                let sequence = {
                    let sequence = state.sequences.entry(key.clone()).or_insert(0);
                    *sequence += 1;
                    *sequence
                };
                state.remove_replication_times(object, &key.1);
                state.remove_interpolation_samples(object, &key.1);
                Some(SyncDelta::despawn(object, key.1, sequence))
            })
            .collect()
    }
}
