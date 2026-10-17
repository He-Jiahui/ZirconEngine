use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};
use std::path::Path;

use super::{
    asset_watch_event::AssetWatchEvent,
    watched_asset_uri_for_path::{is_ignored_watch_path, watched_asset_uri_for_path},
};

#[derive(Default)]
pub(super) struct MappedWatchEvents {
    pub(super) events: Vec<AssetWatchEvent>,
    pub(super) requires_reconciliation: bool,
}

// TODO: [CR-ASSET-PIPELINE-0005] 核实跨平台的重命名配对与重放；无法可靠映射的重命名先请求全量对账。
pub(super) fn map_notify_event(assets_root: &Path, event: Event) -> MappedWatchEvents {
    match event.kind {
        EventKind::Create(_) => {
            map_paths_with_capacity(assets_root, &event.paths, AssetWatchEvent::Added)
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            if let [from, to] = event.paths.as_slice() {
                if let (Ok(from), Ok(to)) = (
                    watched_asset_uri_for_path(assets_root, from),
                    watched_asset_uri_for_path(assets_root, to),
                ) {
                    return MappedWatchEvents {
                        events: vec![AssetWatchEvent::Renamed { from, to }],
                        requires_reconciliation: false,
                    };
                }
            }
            uncertain_rename(assets_root, &event.paths)
        }
        EventKind::Modify(ModifyKind::Name(_)) => uncertain_rename(assets_root, &event.paths),
        EventKind::Modify(_) => map_modified_paths_with_capacity(assets_root, &event.paths),
        EventKind::Remove(_) => {
            map_paths_with_capacity(assets_root, &event.paths, AssetWatchEvent::Removed)
        }
        _ => MappedWatchEvents::default(),
    }
}

fn is_relevant_watch_path(assets_root: &Path, path: &Path) -> bool {
    path.starts_with(assets_root) && !is_ignored_watch_path(path)
}

fn uncertain_rename(assets_root: &Path, paths: &[std::path::PathBuf]) -> MappedWatchEvents {
    MappedWatchEvents {
        events: Vec::new(),
        requires_reconciliation: paths.is_empty()
            || paths
                .iter()
                .any(|path| is_relevant_watch_path(assets_root, path)),
    }
}

fn map_modified_paths_with_capacity(
    assets_root: &Path,
    paths: &[std::path::PathBuf],
) -> MappedWatchEvents {
    let mut events = Vec::with_capacity(paths.len());
    let mut requires_reconciliation = paths.is_empty();
    for path in paths {
        if is_ordinary_directory_modify_echo(path) {
            continue;
        }
        match watched_asset_uri_for_path(assets_root, path) {
            Ok(uri) => events.push(AssetWatchEvent::Modified(uri)),
            Err(_) => requires_reconciliation |= is_relevant_watch_path(assets_root, path),
        }
    }
    MappedWatchEvents {
        events,
        requires_reconciliation,
    }
}

fn is_ordinary_directory_modify_echo(path: &Path) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if !metadata.file_type().is_dir() {
        return false;
    }
    let Some(file_name) = path.file_name() else {
        return false;
    };
    let mut compound_sidecar_name = file_name.to_os_string();
    compound_sidecar_name.push(".zmeta");
    !path.with_file_name(compound_sidecar_name).is_file()
}

fn map_paths_with_capacity<F>(
    assets_root: &Path,
    paths: &[std::path::PathBuf],
    map: F,
) -> MappedWatchEvents
where
    F: Fn(crate::asset::AssetUri) -> AssetWatchEvent,
{
    let mut events = Vec::with_capacity(paths.len());
    let mut requires_reconciliation = paths.is_empty();
    for path in paths {
        match watched_asset_uri_for_path(assets_root, path) {
            Ok(uri) => events.push(map(uri)),
            Err(_) => requires_reconciliation |= is_relevant_watch_path(assets_root, path),
        }
    }
    MappedWatchEvents {
        events,
        requires_reconciliation,
    }
}

#[cfg(test)]
#[path = "tests/map_notify_event_perf.rs"]
mod optimization_batch_20260830bq_runtime_tests;
