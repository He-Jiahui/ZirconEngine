use crossbeam_channel::bounded;
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;

use super::{
    asset_watch_batch::AssetWatchBatch,
    asset_watch_error::AssetWatchError,
    asset_watcher::AssetWatcher,
    asset_watcher::AssetWatcherOptions,
    recommended_watcher::recommended_watcher,
    watch_io_error::watch_io_error,
    watch_loop::{watch_ingress, watch_loop},
};

impl AssetWatcher {
    pub fn spawn(
        assets_root: PathBuf,
        on_changes: impl Fn(AssetWatchBatch) + Send + Sync + 'static,
        on_error: impl Fn(AssetWatchError) + Send + Sync + 'static,
    ) -> Result<Self, std::io::Error> {
        Self::spawn_with_options(
            assets_root,
            AssetWatcherOptions::default(),
            on_changes,
            on_error,
        )
    }

    pub fn spawn_with_options(
        assets_root: PathBuf,
        options: AssetWatcherOptions,
        on_changes: impl Fn(AssetWatchBatch) + Send + Sync + 'static,
        on_error: impl Fn(AssetWatchError) + Send + Sync + 'static,
    ) -> Result<Self, std::io::Error> {
        let (stop_tx, stop_rx) = bounded(1);
        let (ready_tx, ready_rx) = bounded(1);
        let callback = Arc::new(on_changes);
        let error_callback = Arc::new(on_error);
        let join = std::thread::Builder::new()
            .name("zircon-asset-watcher".to_string())
            .spawn(move || {
                let (ingress_tx, ingress_rx) = watch_ingress(options);
                let watcher = recommended_watcher(move |result| {
                    ingress_tx.try_send(result);
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
                watch_loop(
                    assets_root,
                    options,
                    stop_rx,
                    ingress_rx,
                    callback,
                    error_callback,
                );
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
