use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};
use std::path::Path;

use super::{
    asset_watch_event::AssetWatchEvent, watched_asset_uri_for_path::watched_asset_uri_for_path,
};

pub(super) fn map_notify_event(assets_root: &Path, event: Event) -> Vec<AssetWatchEvent> {
    match event.kind {
        EventKind::Create(_) => event
            .paths
            .into_iter()
            .filter_map(|path| watched_asset_uri_for_path(assets_root, &path).ok())
            .map(AssetWatchEvent::Added)
            .collect(),
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
        EventKind::Modify(_) => event
            .paths
            .into_iter()
            .filter_map(|path| watched_asset_uri_for_path(assets_root, &path).ok())
            .map(AssetWatchEvent::Modified)
            .collect(),
        EventKind::Remove(_) => event
            .paths
            .into_iter()
            .filter_map(|path| watched_asset_uri_for_path(assets_root, &path).ok())
            .map(AssetWatchEvent::Removed)
            .collect(),
        _ => Vec::new(),
    }
}
