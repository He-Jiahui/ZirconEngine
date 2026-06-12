use crossbeam_channel::Sender;
use std::thread::JoinHandle;
use std::time::Duration;

pub const ASSET_WATCH_DEFAULT_DEBOUNCE: Duration = Duration::from_millis(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetWatcherOptions {
    pub debounce: Duration,
}

impl Default for AssetWatcherOptions {
    fn default() -> Self {
        Self {
            debounce: ASSET_WATCH_DEFAULT_DEBOUNCE,
        }
    }
}

#[derive(Debug)]
pub struct AssetWatcher {
    pub(super) stop_tx: Sender<()>,
    pub(super) join: Option<JoinHandle<()>>,
}
