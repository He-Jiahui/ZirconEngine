use std::collections::HashMap;

use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncComponentDescriptor, SyncInterestDescriptor, SyncObjectSnapshot,
};

#[derive(Clone, Debug)]
pub(in crate::manager) struct NetReplicationInterpolationSample {
    /// Receive-time sample used by transform interpolation queries.
    pub time_ms: u64,
    pub bytes: Vec<u8>,
}

type NetReplicationFieldSamples = HashMap<String, Vec<NetReplicationInterpolationSample>>;
type NetReplicationObjectSamples = HashMap<NetObjectId, NetReplicationFieldSamples>;

#[derive(Debug, Default)]
pub(in crate::manager) struct NetReplicationRuntimeState {
    pub(in crate::manager) descriptors: HashMap<String, SyncComponentDescriptor>,
    pub(in crate::manager) snapshots: HashMap<(NetObjectId, String), SyncObjectSnapshot>,
    pub(in crate::manager) sequences: HashMap<(NetObjectId, String), u64>,
    pub(in crate::manager) interests: HashMap<NetSessionId, SyncInterestDescriptor>,
    pub(in crate::manager) last_replication_ms: HashMap<(NetSessionId, NetObjectId, String), u64>,
    pub(in crate::manager) interpolation_samples: HashMap<String, NetReplicationObjectSamples>,
}

impl NetReplicationRuntimeState {
    pub(in crate::manager) fn remove_interpolation_samples(
        &mut self,
        object: NetObjectId,
        component_type: &str,
    ) {
        let remove_component = self
            .interpolation_samples
            .get_mut(component_type)
            .is_some_and(|objects| {
                objects.remove(&object);
                objects.is_empty()
            });
        if remove_component {
            self.interpolation_samples.remove(component_type);
        }
    }
}
