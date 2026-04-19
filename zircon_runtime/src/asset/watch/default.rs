use crossbeam_channel::unbounded;

use super::asset_watcher::AssetWatcher;

impl Default for AssetWatcher {
    fn default() -> Self {
        let (stop_tx, _stop_rx) = unbounded();
        Self {
            stop_tx,
            join: None,
        }
    }
}
