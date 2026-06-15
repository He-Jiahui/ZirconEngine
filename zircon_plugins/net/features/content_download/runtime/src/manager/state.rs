use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    NetDownloadId, NetDownloadManifest, NetDownloadProgress, NetManager,
};

#[derive(Clone, Default)]
pub struct NetContentDownloadRuntimeManager {
    pub(in crate::manager) state: Arc<Mutex<NetContentDownloadRuntimeState>>,
    pub(in crate::manager) net: Option<Arc<dyn NetManager>>,
}

impl std::fmt::Debug for NetContentDownloadRuntimeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetContentDownloadRuntimeManager")
            .field("has_net_manager", &self.net.is_some())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
pub(in crate::manager) struct NetContentDownloadRuntimeState {
    pub(in crate::manager) manifests: HashMap<NetDownloadId, NetDownloadManifest>,
    pub(in crate::manager) progress: HashMap<NetDownloadId, NetDownloadProgress>,
    pub(in crate::manager) cache_hits: HashMap<NetDownloadId, Vec<String>>,
    pub(in crate::manager) attempt_indices: HashMap<(NetDownloadId, String), usize>,
    pub(in crate::manager) failed_attempts: HashMap<(NetDownloadId, String), Vec<String>>,
    pub(in crate::manager) partial_chunks: HashMap<(NetDownloadId, String), Vec<u8>>,
    pub(in crate::manager) resume_bitmaps: HashMap<NetDownloadId, Vec<bool>>,
}

impl NetContentDownloadRuntimeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_net_manager(net: Arc<dyn NetManager>) -> Self {
        Self {
            state: Arc::default(),
            net: Some(net),
        }
    }
}
