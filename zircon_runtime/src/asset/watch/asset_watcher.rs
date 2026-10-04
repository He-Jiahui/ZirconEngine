use crossbeam_channel::Sender;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::core::runtime::tasks::thread_is_join_ready;

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

impl AssetWatcher {
    /// Requests stop and joins the watcher only when it reaches the caller's deadline.
    pub fn shutdown_until(&mut self, deadline: Instant) -> bool {
        let _ = self.stop_tx.try_send(());
        let Some(join) = self.join.as_ref() else {
            return true;
        };
        while !thread_is_join_ready(join) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            std::thread::sleep(Duration::from_millis(1).min(remaining));
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn from_test_worker(stop_tx: Sender<()>, join: JoinHandle<()>) -> Self {
        Self {
            stop_tx,
            join: Some(join),
        }
    }
}
