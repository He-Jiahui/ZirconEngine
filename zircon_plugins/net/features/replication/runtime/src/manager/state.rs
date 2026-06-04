use std::collections::HashMap;

use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncComponentDescriptor, SyncInterestDescriptor, SyncObjectSnapshot,
};

#[derive(Debug, Default)]
pub(in crate::manager) struct NetReplicationRuntimeState {
    pub(in crate::manager) descriptors: HashMap<String, SyncComponentDescriptor>,
    pub(in crate::manager) snapshots: HashMap<(NetObjectId, String), SyncObjectSnapshot>,
    pub(in crate::manager) sequences: HashMap<(NetObjectId, String), u64>,
    pub(in crate::manager) interests: HashMap<NetSessionId, SyncInterestDescriptor>,
    pub(in crate::manager) last_replication_ms: HashMap<(NetSessionId, NetObjectId, String), u64>,
}
