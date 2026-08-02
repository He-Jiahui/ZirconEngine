use crossbeam_channel::Sender;
use std::thread::JoinHandle;
use std::time::Duration;

pub const ASSET_WATCH_DEFAULT_DEBOUNCE: Duration = Duration::from_millis(120);
pub const ASSET_WATCH_DEFAULT_MAX_BATCH_LATENCY: Duration = Duration::from_millis(500);
pub const ASSET_WATCH_DEFAULT_INGRESS_ENTRY_CAPACITY: usize = 1_024;
pub const ASSET_WATCH_DEFAULT_INGRESS_BYTE_CAPACITY: usize = 2 * 1024 * 1024;
pub const ASSET_WATCH_DEFAULT_PENDING_ENTRY_CAPACITY: usize = 4_096;
pub const ASSET_WATCH_DEFAULT_PENDING_BYTE_CAPACITY: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetWatcherOptions {
    pub debounce: Duration,
    pub max_batch_latency: Duration,
    pub ingress_entry_capacity: usize,
    pub ingress_byte_capacity: usize,
    pub pending_entry_capacity: usize,
    pub pending_byte_capacity: usize,
}

impl Default for AssetWatcherOptions {
    fn default() -> Self {
        Self {
            debounce: ASSET_WATCH_DEFAULT_DEBOUNCE,
            max_batch_latency: ASSET_WATCH_DEFAULT_MAX_BATCH_LATENCY,
            ingress_entry_capacity: ASSET_WATCH_DEFAULT_INGRESS_ENTRY_CAPACITY,
            ingress_byte_capacity: ASSET_WATCH_DEFAULT_INGRESS_BYTE_CAPACITY,
            pending_entry_capacity: ASSET_WATCH_DEFAULT_PENDING_ENTRY_CAPACITY,
            pending_byte_capacity: ASSET_WATCH_DEFAULT_PENDING_BYTE_CAPACITY,
        }
    }
}

#[derive(Debug)]
pub struct AssetWatcher {
    pub(super) stop_tx: Sender<()>,
    pub(super) join: Option<JoinHandle<()>>,
}
