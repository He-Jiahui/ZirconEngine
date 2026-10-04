use std::sync::Arc;

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogSnapshotRecord, EditorAssetChangeKind,
    EditorAssetChangeRecord,
};

use super::{lock_editor_asset_gate_recovering_poison, DefaultEditorAssetManager};

impl DefaultEditorAssetManager {
    /// Replaces the active runtime projection with an empty, newer catalog generation.
    ///
    /// Invalidating the source-sync epoch prevents an older project snapshot from committing;
    /// replacing the catalog generation and preview scheduler separately rejects old preview
    /// completions after the close transition.
    pub fn deactivate_runtime_project(&self) -> bool {
        let _source_sync_guard =
            lock_editor_asset_gate_recovering_poison(self.source_sync_gate.as_ref());
        self.advance_source_sync_epoch();
        self.clear_import_flow();

        let change = {
            let _publish_guard =
                lock_editor_asset_gate_recovering_poison(self.publish_gate.as_ref());
            let mut state = self.write_state_recovering_poison();
            if runtime_project_projection_is_empty(&state) {
                return false;
            }

            let (catalog_revision, publish_epoch) =
                state.catalog_generation.next_catalog_identity();
            let catalog_generation = Arc::new(EditorAssetCatalogGeneration::from_snapshot_record(
                EditorAssetCatalogSnapshotRecord {
                    catalog_revision,
                    ..EditorAssetCatalogSnapshotRecord::default()
                },
                publish_epoch,
            ));
            let mut cleared = super::EditorAssetState::default();
            cleared.catalog_generation = catalog_generation;
            *state = cleared;

            EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::CatalogChanged,
                catalog_revision,
                uuid: None,
                locator: None,
            }
        };
        self.broadcast(change);
        true
    }
}

fn runtime_project_projection_is_empty(state: &super::EditorAssetState) -> bool {
    state.project_root.is_none()
        && state.assets_root.is_none()
        && state.cache_root.is_none()
        && state.project_name.is_empty()
        && state.default_scene_uri.is_none()
        && state.project.is_none()
        && state.catalog_generation.project_root.is_empty()
        && state.catalog_generation.assets.is_empty()
        && state.catalog_generation.folders.is_empty()
        && state.asset_index.is_none()
        && state.preview_cache.is_none()
}

#[cfg(test)]
#[path = "tests/project_deactivation.rs"]
mod tests;
