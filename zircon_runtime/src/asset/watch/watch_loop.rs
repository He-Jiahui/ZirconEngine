use crossbeam_channel::{after, select, Receiver};
use notify::Event;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::{
    asset_change::AssetChange,
    asset_watch_error::AssetWatchError,
    asset_watcher::AssetWatcherOptions,
    fold_events::{finish_folded_events, fold_event, FoldedAssetChangeMap},
    map_notify_event::map_notify_event,
};

pub(super) fn watch_loop(
    assets_root: PathBuf,
    options: AssetWatcherOptions,
    stop_rx: Receiver<()>,
    event_rx: Receiver<notify::Result<Event>>,
    on_changes: Arc<dyn Fn(Vec<AssetChange>) + Send + Sync>,
    on_error: Arc<dyn Fn(AssetWatchError) + Send + Sync>,
) {
    watch_loop_inner(
        assets_root,
        options.debounce,
        stop_rx,
        event_rx,
        on_changes,
        on_error,
    );
}

#[cfg(test)]
pub(crate) fn watch_loop_for_test(
    assets_root: PathBuf,
    debounce: Duration,
    stop_rx: Receiver<()>,
    event_rx: Receiver<notify::Result<Event>>,
    on_changes: Arc<dyn Fn(Vec<AssetChange>) + Send + Sync>,
    on_error: Arc<dyn Fn(AssetWatchError) + Send + Sync>,
) {
    watch_loop_inner(
        assets_root,
        debounce,
        stop_rx,
        event_rx,
        on_changes,
        on_error,
    );
}

fn watch_loop_inner(
    assets_root: PathBuf,
    debounce: Duration,
    stop_rx: Receiver<()>,
    event_rx: Receiver<notify::Result<Event>>,
    on_changes: Arc<dyn Fn(Vec<AssetChange>) + Send + Sync>,
    on_error: Arc<dyn Fn(AssetWatchError) + Send + Sync>,
) {
    loop {
        select! {
            recv(stop_rx) -> _ => break,
            recv(event_rx) -> message => match message {
                Ok(Ok(event)) => {
                    let mut pending = FoldedAssetChangeMap::new();
                    for event in map_notify_event(&assets_root, event) {
                        fold_event(&mut pending, event);
                    }
                    if pending.is_empty() {
                        continue;
                    }
                    loop {
                        select! {
                            recv(stop_rx) -> _ => return,
                            recv(event_rx) -> next => match next {
                                Ok(Ok(event)) => {
                                    for event in map_notify_event(&assets_root, event) {
                                        fold_event(&mut pending, event);
                                    }
                                }
                                Ok(Err(error)) => on_error(AssetWatchError::from_notify_error(assets_root.clone(), error)),
                                Err(_) => return,
                            },
                            recv(after(debounce)) -> _ => break,
                        }
                    }
                    let folded = finish_folded_events(pending);
                    if !folded.is_empty() {
                        on_changes(folded);
                    }
                }
                Ok(Err(error)) => on_error(AssetWatchError::from_notify_error(assets_root.clone(), error)),
                Err(_) => break,
            }
        }
    }
}
