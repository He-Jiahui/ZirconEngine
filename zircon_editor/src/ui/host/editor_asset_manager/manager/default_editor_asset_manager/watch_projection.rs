use std::collections::BTreeMap;

use zircon_runtime::asset::watch::{AssetChange, AssetChangeKind, AssetWatchEvent};

use super::super::catalog_generation::update_catalog_records_in_catalog_generation;
use super::{lock_editor_asset_gate_recovering_poison, DefaultEditorAssetManager};
use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetChangeKind, EditorAssetChangeRecord,
};

impl DefaultEditorAssetManager {
    /// Projects raw Runtime watcher changes into the active immutable catalog generation.
    ///
    /// Runtime remains responsible for import and registry publication. This method only makes
    /// current editor rows stale immediately, then the ordinary Runtime refresh replaces those
    /// rows with the committed authoritative generation.
    pub fn project_runtime_asset_changes(&self, changes: &[AssetChange]) {
        zircon_runtime::profile_scope!("editor", "asset_catalog", "watch_projection");
        let watch_events = asset_watch_events(changes);
        if watch_events.is_empty() {
            return;
        }

        let (changes, dirty_uuid_count, pending_path_count, row_update_count) = {
            let _publish_guard =
                lock_editor_asset_gate_recovering_poison(self.publish_gate.as_ref());
            let mut state = self.write_state_recovering_poison();
            let Some(asset_index) = state.asset_index.as_ref() else {
                return;
            };
            let mut asset_index = asset_index
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            asset_index.apply_watch_events(&watch_events);
            let dirty_uuid_count = asset_index.dirty_count();
            let pending_path_count = asset_index.pending_dirty_path_count();
            let updates = dirty_catalog_records(&state.catalog_generation, &watch_events);
            drop(asset_index);
            if updates.is_empty() {
                (Vec::new(), dirty_uuid_count, pending_path_count, 0)
            } else {
                let publish_epoch = state.catalog_generation.publish_epoch.saturating_add(1);
                state.catalog_generation = update_catalog_records_in_catalog_generation(
                    &state.catalog_generation,
                    updates.values().cloned(),
                    publish_epoch,
                );
                let row_update_count = updates.len();
                let changes = updates
                    .values()
                    .map(|record| EditorAssetChangeRecord {
                        kind: EditorAssetChangeKind::AssetStateChanged,
                        catalog_revision: state.catalog_generation.catalog_revision,
                        uuid: Some(record.asset_uuid.to_string()),
                        locator: Some(record.locator.to_string()),
                    })
                    .collect::<Vec<_>>();
                (
                    changes,
                    dirty_uuid_count,
                    pending_path_count,
                    row_update_count,
                )
            }
        };
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.watch_state_event_count",
            watch_events.len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.watch_state_row_update_count",
            row_update_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.watch_index_dirty_uuid_count",
            dirty_uuid_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.watch_index_pending_path_count",
            pending_path_count
        );

        for change in changes {
            self.broadcast(change);
        }
    }
}

fn asset_watch_events(changes: &[AssetChange]) -> Vec<AssetWatchEvent> {
    changes
        .iter()
        .map(|change| match &change.kind {
            AssetChangeKind::Added => AssetWatchEvent::Added(change.uri.clone()),
            AssetChangeKind::Modified => AssetWatchEvent::Modified(change.uri.clone()),
            AssetChangeKind::Removed => AssetWatchEvent::Removed(change.uri.clone()),
            AssetChangeKind::Renamed => match change.previous_uri.as_ref() {
                Some(previous_uri) => AssetWatchEvent::Renamed {
                    from: previous_uri.clone(),
                    to: change.uri.clone(),
                },
                None => AssetWatchEvent::Added(change.uri.clone()),
            },
        })
        .collect()
}

fn dirty_catalog_records(
    catalog: &crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration,
    events: &[AssetWatchEvent],
) -> BTreeMap<String, AssetCatalogRecord> {
    let mut updates = BTreeMap::new();
    for event in events {
        match event {
            AssetWatchEvent::Added(uri)
            | AssetWatchEvent::Modified(uri)
            | AssetWatchEvent::Removed(uri) => {
                mark_catalog_record_dirty(catalog, uri, &mut updates)
            }
            AssetWatchEvent::Renamed { from, to } => {
                mark_catalog_record_dirty(catalog, from, &mut updates);
                mark_catalog_record_dirty(catalog, to, &mut updates);
            }
        }
    }
    updates
}

fn mark_catalog_record_dirty(
    catalog: &crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration,
    uri: &zircon_runtime::asset::AssetUri,
    updates: &mut BTreeMap<String, AssetCatalogRecord>,
) {
    let Some(asset) = catalog.asset_by_locator(&uri.to_string()) else {
        return;
    };
    let Some(current) = catalog.catalog_record(&asset.uuid) else {
        return;
    };
    if current.dirty {
        return;
    }
    let mut updated = (*current).clone();
    updated.dirty = true;
    updates.insert(updated.asset_uuid.to_string(), updated);
}

#[cfg(test)]
mod tests {
    use zircon_runtime::asset::watch::{AssetChange, AssetChangeKind, AssetWatchEvent};
    use zircon_runtime::asset::AssetUri;

    use super::asset_watch_events;

    #[test]
    fn incomplete_runtime_rename_is_a_safe_added_event() {
        let uri = AssetUri::parse("res://models/ship.glb").unwrap();
        assert_eq!(
            asset_watch_events(&[AssetChange::new(
                AssetChangeKind::Renamed,
                uri.clone(),
                None
            )]),
            vec![AssetWatchEvent::Added(uri)]
        );
    }
}
