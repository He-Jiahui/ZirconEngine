//! 比较同一对象/组件的新旧字段，生成变更 delta 并保存最新快照。
//! 只有已登记组件会被接受；调用者须提供完整一致的字段集，缺失字段的删除语义未在此定义。

use zircon_runtime::core::framework::net::{
    NetObjectId, SyncDelta, SyncFieldValue, SyncObjectSnapshot,
};

use super::NetReplicationRuntimeManager;

impl NetReplicationRuntimeManager {
    pub(in crate::manager) fn publish_snapshot_impl(
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
}

pub(in crate::manager) fn snapshot_payload_bytes(snapshot: &SyncObjectSnapshot) -> usize {
    snapshot.fields.iter().map(|field| field.bytes.len()).sum()
}
