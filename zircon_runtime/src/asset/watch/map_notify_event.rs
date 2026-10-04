use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};
use std::path::Path;

use super::{
    asset_watch_event::AssetWatchEvent, watched_asset_uri_for_path::watched_asset_uri_for_path,
};

// TODO: [CR-ASSET-PIPELINE-0005] 核实跨资产根及 From/To 分拆重命名在各平台的 notify 事件形态；当前无法映射 Both 任一端时静默丢弃，可能缺少退役或导入。
pub(super) fn map_notify_event(assets_root: &Path, event: Event) -> Vec<AssetWatchEvent> {
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
                    return vec![AssetWatchEvent::Renamed { from, to }];
                }
            }
            Vec::new()
        }
        EventKind::Modify(_) => map_modified_paths_with_capacity(assets_root, &event.paths),
        EventKind::Remove(_) => {
            map_paths_with_capacity(assets_root, &event.paths, AssetWatchEvent::Removed)
        }
        _ => Vec::new(),
    }
}

fn map_modified_paths_with_capacity(
    assets_root: &Path,
    paths: &[std::path::PathBuf],
) -> Vec<AssetWatchEvent> {
    let mut events = Vec::with_capacity(paths.len());
    for path in paths {
        if is_ordinary_directory_modify_echo(path) {
            continue;
        }
        if let Ok(uri) = watched_asset_uri_for_path(assets_root, path) {
            events.push(AssetWatchEvent::Modified(uri));
        }
    }
    events
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
) -> Vec<AssetWatchEvent>
where
    F: Fn(crate::asset::AssetUri) -> AssetWatchEvent,
{
    let mut events = Vec::with_capacity(paths.len());
    for path in paths {
        if let Ok(uri) = watched_asset_uri_for_path(assets_root, path) {
            events.push(map(uri));
        }
    }
    events
}

#[cfg(test)]
#[path = "tests/map_notify_event_optimization_batch_20260830bq_runtime_tests.rs"]
mod optimization_batch_20260830bq_runtime_tests;
