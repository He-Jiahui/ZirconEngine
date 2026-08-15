mod asset_change;
mod asset_change_construction;
mod asset_change_kind;
mod asset_uri_for_path;
mod asset_watch_batch;
mod asset_watch_error;
mod asset_watch_event;
mod asset_watcher;
mod default;
mod fold_events;
mod is_meta_sidecar;
mod map_notify_event;
mod recommended_watcher;
mod shutdown_on_drop;
mod spawn;
mod watch_io_error;
mod watch_loop;
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
