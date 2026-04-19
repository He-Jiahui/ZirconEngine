use std::collections::BTreeMap;

use super::{
    asset_change::AssetChange, asset_change_kind::AssetChangeKind,
    asset_watch_event::AssetWatchEvent, asset_watcher::AssetWatcher,
};

impl AssetWatcher {
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
