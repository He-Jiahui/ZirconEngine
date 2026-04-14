use crossbeam_channel::{after, select, unbounded, Receiver, Sender};
use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::AssetUri;

const WATCH_DEBOUNCE: Duration = Duration::from_millis(120);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetWatchEvent {
    Added(AssetUri),
    Modified(AssetUri),
    Removed(AssetUri),
    Renamed { from: AssetUri, to: AssetUri },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetChangeKind {
    Added,
    Modified,
    Removed,
    Renamed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetChange {
    pub kind: AssetChangeKind,
    pub uri: AssetUri,
    pub previous_uri: Option<AssetUri>,
}

impl AssetChange {
    pub fn new(kind: AssetChangeKind, uri: AssetUri, previous_uri: Option<AssetUri>) -> Self {
        Self {
            kind,
            uri,
            previous_uri,
        }
    }
}

#[derive(Debug)]
pub struct AssetWatcher {
    stop_tx: Sender<()>,
    join: Option<JoinHandle<()>>,
}

impl Default for AssetWatcher {
    fn default() -> Self {
        let (stop_tx, _stop_rx) = unbounded();
        Self {
            stop_tx,
            join: None,
        }
    }
}

impl AssetWatcher {
    pub fn spawn(
        assets_root: PathBuf,
        on_changes: impl Fn(Vec<AssetChange>) + Send + Sync + 'static,
    ) -> Result<Self, std::io::Error> {
        let (stop_tx, stop_rx) = unbounded();
        let (ready_tx, ready_rx) = unbounded();
        let callback = std::sync::Arc::new(on_changes);
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

    pub fn fold_events(events: &[AssetWatchEvent]) -> Vec<AssetChange> {
        let mut folded = BTreeMap::<String, AssetChange>::new();

        for event in events {
            match event {
                AssetWatchEvent::Added(uri) => {
                    folded.insert(
                        uri.to_string(),
                        AssetChange::new(AssetChangeKind::Added, uri.clone(), None),
                    );
                }
                AssetWatchEvent::Modified(uri) => {
                    folded
                        .entry(uri.to_string())
                        .and_modify(|change| {
                            if !matches!(change.kind, AssetChangeKind::Added) {
                                change.kind = AssetChangeKind::Modified;
                            }
                        })
                        .or_insert_with(|| {
                            AssetChange::new(AssetChangeKind::Modified, uri.clone(), None)
                        });
                }
                AssetWatchEvent::Removed(uri) => {
                    folded.insert(
                        uri.to_string(),
                        AssetChange::new(AssetChangeKind::Removed, uri.clone(), None),
                    );
                }
                AssetWatchEvent::Renamed { from, to } => {
                    folded.remove(&from.to_string());
                    folded.insert(
                        to.to_string(),
                        AssetChange::new(AssetChangeKind::Renamed, to.clone(), Some(from.clone())),
                    );
                }
            }
        }

        folded.into_values().collect()
    }
}

impl Drop for AssetWatcher {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn watch_loop(
    assets_root: PathBuf,
    stop_rx: Receiver<()>,
    event_rx: Receiver<notify::Result<Event>>,
    on_changes: std::sync::Arc<dyn Fn(Vec<AssetChange>) + Send + Sync>,
) {
    loop {
        select! {
            recv(stop_rx) -> _ => break,
            recv(event_rx) -> message => match message {
                Ok(Ok(event)) => {
                    let mut pending = map_notify_event(&assets_root, event);
                    if pending.is_empty() {
                        continue;
                    }
                    loop {
                        select! {
                            recv(stop_rx) -> _ => return,
                            recv(event_rx) -> next => match next {
                                Ok(Ok(event)) => pending.extend(map_notify_event(&assets_root, event)),
                                Ok(Err(_)) => {}
                                Err(_) => return,
                            },
                            recv(after(WATCH_DEBOUNCE)) -> _ => break,
                        }
                    }
                    let folded = AssetWatcher::fold_events(&pending);
                    if !folded.is_empty() {
                        on_changes(folded);
                    }
                }
                Ok(Err(_)) => {}
                Err(_) => break,
            }
        }
    }
}

fn map_notify_event(assets_root: &Path, event: Event) -> Vec<AssetWatchEvent> {
    match event.kind {
        EventKind::Create(_) => event
            .paths
            .into_iter()
            .filter_map(|path| asset_uri_for_path(assets_root, &path).ok())
            .map(AssetWatchEvent::Added)
            .collect(),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            if let [from, to] = event.paths.as_slice() {
                if let (Ok(from), Ok(to)) = (
                    asset_uri_for_path(assets_root, from),
                    asset_uri_for_path(assets_root, to),
                ) {
                    return vec![AssetWatchEvent::Renamed { from, to }];
                }
            }
            Vec::new()
        }
        EventKind::Modify(_) => event
            .paths
            .into_iter()
            .filter_map(|path| asset_uri_for_path(assets_root, &path).ok())
            .map(AssetWatchEvent::Modified)
            .collect(),
        EventKind::Remove(_) => event
            .paths
            .into_iter()
            .filter_map(|path| asset_uri_for_path(assets_root, &path).ok())
            .map(AssetWatchEvent::Removed)
            .collect(),
        _ => Vec::new(),
    }
}

fn asset_uri_for_path(assets_root: &Path, path: &Path) -> Result<AssetUri, crate::AssetUriError> {
    let relative = match path.strip_prefix(assets_root) {
        Ok(relative) => relative,
        Err(_) => return Err(crate::AssetUriError::EscapeAttempt(path.display().to_string())),
    };
    let normalized = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    AssetUri::parse(&format!("res://{normalized}"))
}

fn watch_io_error(error: notify::Error) -> std::io::Error {
    std::io::Error::other(error.to_string())
}

fn recommended_watcher(
    handler: impl FnMut(notify::Result<Event>) + Send + 'static,
) -> Result<RecommendedWatcher, notify::Error> {
    notify::recommended_watcher(handler)
}
