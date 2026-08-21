use std::sync::atomic::Ordering;
use std::sync::Arc;

use zircon_runtime::core::CoreError;

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogSnapshotRecord, EditorAssetChangeKind,
    EditorAssetChangeRecord,
};

use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    /// Replaces the active runtime projection with an empty, newer catalog generation.
    ///
    /// Invalidating the source-sync epoch prevents an older project snapshot from committing;
    /// replacing the catalog generation and preview scheduler separately rejects old preview
    /// completions after the close transition.
    pub fn deactivate_runtime_project(&self) -> Result<bool, CoreError> {
        let _source_sync_guard = self
            .source_sync_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let change = {
            let _publish_guard = self
                .publish_gate
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            if runtime_project_projection_is_empty(&state) {
                return Ok(false);
            }

            self.source_sync_epoch.fetch_add(1, Ordering::AcqRel);
            let catalog_revision = state.catalog_generation.catalog_revision.saturating_add(1);
            let publish_epoch = state.catalog_generation.publish_epoch.saturating_add(1);
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
        Ok(true)
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
        && state.catalog_by_uuid.is_empty()
        && state.uuid_by_locator.is_empty()
        && state.preview_cache.is_none()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
    use zircon_runtime::asset::{AssetUri, AssetUuid};

    use crate::ui::host::editor_asset_manager::{EditorAssetChangeKind, EditorAssetManager};

    use super::super::DefaultEditorAssetManager;

    #[test]
    fn runtime_project_deactivation_replaces_the_catalog_with_a_new_empty_generation() {
        let root = unique_temp_project_root("asset_catalog_project_close");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .unwrap();
        ProjectManifest::new(
            "Close Catalog",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();

        let manager = DefaultEditorAssetManager::new();
        let changes = EditorAssetManager::subscribe_editor_asset_changes(&manager);
        manager
            .sync_from_project(ProjectManager::open(&root).unwrap())
            .unwrap();
        while changes.try_recv().is_some() {}
        let before = manager.catalog_snapshot_record();
        let stale_preview_uuid = AssetUuid::new();
        let (previous_source_generation, stale_preview_token) = {
            let mut state = manager
                .state
                .write()
                .expect("editor asset state lock poisoned");
            state.preview_scheduler.mark_dirty(stale_preview_uuid);
            let stale_preview_token = state
                .preview_scheduler
                .request_refresh(stale_preview_uuid, true)
                .expect("old preview generation admission");
            (Arc::clone(&state.source_generation), stale_preview_token)
        };

        assert!(EditorAssetManager::deactivate_runtime_project(&manager).unwrap());

        let after = manager.catalog_snapshot_record();
        assert!(after.assets.is_empty());
        assert!(after.folders.is_empty());
        assert!(after.project_root.is_empty());
        assert_eq!(after.catalog_revision, before.catalog_revision + 1);
        assert_eq!(after.publish_epoch, before.publish_epoch + 1);
        let change = changes.try_recv().expect("empty catalog change");
        assert_eq!(change.change.kind, EditorAssetChangeKind::CatalogChanged);
        assert_eq!(change.change.catalog_revision, after.catalog_revision);

        let mut state = manager
            .state
            .write()
            .expect("editor asset state lock poisoned");
        assert!(state.project_root.is_none());
        assert!(state.assets_root.is_none());
        assert!(state.cache_root.is_none());
        assert!(state.project_name.is_empty());
        assert!(state.default_scene_uri.is_none());
        assert!(state.project.is_none());
        assert!(state.catalog_by_uuid.is_empty());
        assert!(state.uuid_by_locator.is_empty());
        assert!(state.preview_cache.is_none());
        assert!(!Arc::ptr_eq(
            &state.source_generation,
            &previous_source_generation
        ));
        assert!(!state
            .preview_scheduler
            .complete_refresh(stale_preview_uuid, stale_preview_token));
        drop(state);

        let deactivated_epoch = manager.source_sync_epoch.load(Ordering::Acquire);
        assert!(!EditorAssetManager::deactivate_runtime_project(&manager).unwrap());
        assert_eq!(
            manager.source_sync_epoch.load(Ordering::Acquire),
            deactivated_epoch,
            "a no-op deactivation must not invalidate an in-flight source sync"
        );
        assert!(changes.try_recv().is_none());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_project_deactivation_clears_residual_projection_without_catalog_assets() {
        let manager = DefaultEditorAssetManager::new();
        let changes = EditorAssetManager::subscribe_editor_asset_changes(&manager);
        {
            let mut state = manager
                .state
                .write()
                .expect("editor asset state lock poisoned");
            state.project_root = Some(std::path::PathBuf::from("C:/projects/retired"));
            state.project_name = "Retired Project".to_string();
        }
        let before = manager.catalog_snapshot_record();

        assert!(EditorAssetManager::deactivate_runtime_project(&manager).unwrap());

        let after = manager.catalog_snapshot_record();
        assert_eq!(after.catalog_revision, before.catalog_revision + 1);
        let state = manager
            .state
            .read()
            .expect("editor asset state lock poisoned");
        assert!(state.project_root.is_none());
        assert!(state.project_name.is_empty());
        drop(state);
        assert_eq!(
            changes
                .try_recv()
                .expect("empty catalog change")
                .change
                .kind,
            EditorAssetChangeKind::CatalogChanged
        );
    }

    fn unique_temp_project_root(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zircon_editor_{label}_{}_{}",
            std::process::id(),
            unique
        ))
    }
}
