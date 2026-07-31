use std::collections::BTreeMap;

use crate::asset::AssetUri;

use super::{
    asset_change::AssetChange, asset_change_kind::AssetChangeKind,
    asset_watch_event::AssetWatchEvent, asset_watcher::AssetWatcher,
};

pub(super) type FoldedAssetChangeMap = BTreeMap<AssetUri, (AssetChangeKind, Option<AssetUri>)>;

impl AssetWatcher {
    pub fn fold_events(events: &[AssetWatchEvent]) -> Vec<AssetChange> {
        let mut folded = FoldedAssetChangeMap::new();
        for event in events.iter().cloned() {
            fold_event(&mut folded, event);
        }
        finish_folded_events(folded)
    }
}

pub(super) fn fold_event(folded: &mut FoldedAssetChangeMap, event: AssetWatchEvent) {
    match event {
        AssetWatchEvent::Added(uri) => {
            folded.insert(uri, (AssetChangeKind::Added, None));
        }
        AssetWatchEvent::Modified(uri) => {
            folded
                .entry(uri)
                .and_modify(|change| {
                    if change.0 != AssetChangeKind::Added {
                        change.0 = AssetChangeKind::Modified;
                    }
                })
                .or_insert((AssetChangeKind::Modified, None));
        }
        AssetWatchEvent::Removed(uri) => {
            folded.insert(uri, (AssetChangeKind::Removed, None));
        }
        AssetWatchEvent::Renamed { from, to } => {
            folded.remove(&from);
            folded.insert(to, (AssetChangeKind::Renamed, Some(from)));
        }
    }
}

pub(super) fn finish_folded_events(folded: FoldedAssetChangeMap) -> Vec<AssetChange> {
    folded
        .into_iter()
        .map(|(uri, (kind, previous_uri))| AssetChange::new(kind, uri, previous_uri))
        .collect()
}
