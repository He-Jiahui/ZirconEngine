use crossbeam_channel::bounded;

use super::asset_watcher::AssetWatcher;

impl Default for AssetWatcher {
    fn default() -> Self {
        let (stop_tx, _stop_rx) = bounded(1);
        Self {
            stop_tx,
            join: None,
        }
    }
}
