use crossbeam_channel::bounded;

use super::asset_watcher::AssetWatcher;

// 默认值仅表示尚未启动的 watcher；notify 监听线程由 spawn 创建。
impl Default for AssetWatcher {
    fn default() -> Self {
        let (stop_tx, _stop_rx) = bounded(1);
        Self {
            stop_tx,
            join: None,
        }
    }
}
