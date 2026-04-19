use crossbeam_channel::unbounded;
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;

use super::{
    asset_change::AssetChange, asset_watcher::AssetWatcher,
    recommended_watcher::recommended_watcher, watch_io_error::watch_io_error,
    watch_loop::watch_loop,
};

impl AssetWatcher {
    pub fn spawn(
        assets_root: PathBuf,
        on_changes: impl Fn(Vec<AssetChange>) + Send + Sync + 'static,
    ) -> Result<Self, std::io::Error> {
        let (stop_tx, stop_rx) = unbounded();
        let (ready_tx, ready_rx) = unbounded();
        let callback = Arc::new(on_changes);
        let join = std::thread::Builder::new()
            .name("zircon-asset-watcher".to_string())
            .spawn(move || {
                let (event_tx, event_rx) = unbounded();
                let watcher = recommended_watcher(move |result| {
                    let _ = event_tx.send(result);
                });
                let mut watcher = match watcher {
                    Ok(watcher) => watcher,
                    Err(error) => {
                        let _ = ready_tx.send(Err(watch_io_error(error)));
                        return;
                    }
                };
                if let Err(error) = watcher.watch(&assets_root, RecursiveMode::Recursive) {
                    let _ = ready_tx.send(Err(watch_io_error(error)));
                    return;
                }
                let _ = ready_tx.send(Ok(()));
                watch_loop(assets_root, stop_rx, event_rx, callback);
                drop(watcher);
            })
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        ready_rx
            .recv()
            .map_err(|error| std::io::Error::other(error.to_string()))??;

        Ok(Self {
            stop_tx,
            join: Some(join),
        })
    }
}
