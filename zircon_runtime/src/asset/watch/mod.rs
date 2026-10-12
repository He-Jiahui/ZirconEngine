#[path = "asset_change.rs"]
mod asset_change;
#[path = "asset_change_construction.rs"]
mod asset_change_construction;
#[path = "asset_change_kind.rs"]
mod asset_change_kind;
#[path = "asset_uri_for_path.rs"]
mod asset_uri_for_path;
#[path = "asset_watch_batch.rs"]
mod asset_watch_batch;
#[path = "asset_watch_error.rs"]
mod asset_watch_error;
#[path = "asset_watch_event.rs"]
mod asset_watch_event;
#[path = "asset_watcher.rs"]
mod asset_watcher;
#[path = "default.rs"]
mod default;
#[path = "fold_events.rs"]
mod fold_events;
#[path = "is_meta_sidecar.rs"]
mod is_meta_sidecar;
#[path = "map_notify_event.rs"]
mod map_notify_event;
#[path = "recommended_watcher.rs"]
mod recommended_watcher;
#[path = "shutdown_on_drop.rs"]
mod shutdown_on_drop;
#[path = "spawn.rs"]
mod spawn;
#[path = "watch_io_error.rs"]
mod watch_io_error;
#[path = "watch_loop.rs"]
mod watch_loop;
#[path = "watched_asset_uri_for_path.rs"]
mod watched_asset_uri_for_path;

pub use asset_change::AssetChange;
pub use asset_change_kind::AssetChangeKind;
pub use asset_watch_batch::{AssetWatchBatch, AssetWatchBatchDiagnostics};
pub use asset_watch_error::AssetWatchError;
pub use asset_watch_event::AssetWatchEvent;
pub use asset_watcher::{
    AssetWatcher, AssetWatcherOptions, ASSET_WATCH_DEFAULT_DEBOUNCE,
    ASSET_WATCH_DEFAULT_INGRESS_BYTE_CAPACITY, ASSET_WATCH_DEFAULT_INGRESS_ENTRY_CAPACITY,
    ASSET_WATCH_DEFAULT_MAX_BATCH_LATENCY, ASSET_WATCH_DEFAULT_PENDING_BYTE_CAPACITY,
    ASSET_WATCH_DEFAULT_PENDING_ENTRY_CAPACITY,
};
#[cfg(test)]
pub(crate) use watch_loop::{watch_ingress, watch_loop_for_test};
#[cfg(test)]
pub(crate) use watched_asset_uri_for_path::watched_asset_uri_for_path;
