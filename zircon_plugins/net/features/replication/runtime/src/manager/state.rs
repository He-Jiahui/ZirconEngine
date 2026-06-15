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

#[derive(Debug, Default)]
pub(in crate::manager) struct NetReplicationRuntimeState {
    pub(in crate::manager) descriptors: HashMap<String, SyncComponentDescriptor>,
    pub(in crate::manager) snapshots: HashMap<(NetObjectId, String), SyncObjectSnapshot>,
    pub(in crate::manager) sequences: HashMap<(NetObjectId, String), u64>,
    pub(in crate::manager) interests: HashMap<NetSessionId, SyncInterestDescriptor>,
    pub(in crate::manager) last_replication_ms: HashMap<(NetSessionId, NetObjectId, String), u64>,
    pub(in crate::manager) interpolation_samples:
        HashMap<(NetObjectId, String, String), Vec<NetReplicationInterpolationSample>>,
}

impl NetReplicationRuntimeState {
    pub(in crate::manager) fn remove_interpolation_samples(
        &mut self,
        object: NetObjectId,
        component_type: &str,
    ) {
        self.interpolation_samples
            .retain(|(sample_object, sample_component, _), _| {
                *sample_object != object || sample_component != component_type
            });
    }
}
