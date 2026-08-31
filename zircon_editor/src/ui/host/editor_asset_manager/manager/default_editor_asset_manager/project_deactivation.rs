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
        let stale_preview_token = {
            let mut state = manager
                .state
                .write()
                .expect("editor asset state lock poisoned");
            state.preview_scheduler.mark_dirty(stale_preview_uuid);
            let stale_preview_token = state
                .preview_scheduler
                .request_refresh(stale_preview_uuid, true)
                .expect("old preview generation admission");
            stale_preview_token
        };

        assert!(EditorAssetManager::deactivate_runtime_project(&manager));

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
        assert!(state.asset_index.is_none());
        assert!(state.preview_cache.is_none());
        assert!(!state
            .preview_scheduler
            .complete_refresh(stale_preview_uuid, stale_preview_token));
        drop(state);

        let deactivated_epoch = manager.source_sync_epoch.load(Ordering::Acquire);
        assert!(!EditorAssetManager::deactivate_runtime_project(&manager));
        assert_eq!(
            manager.source_sync_epoch.load(Ordering::Acquire),
            deactivated_epoch + 1,
            "a no-op projection retirement must still invalidate registered source sync work"
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

        assert!(EditorAssetManager::deactivate_runtime_project(&manager));

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

    #[test]
    fn runtime_project_deactivation_clears_a_residual_runtime_asset_index() {
        let root = unique_temp_project_root("asset_catalog_residual_runtime_asset_index");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .unwrap();
        ProjectManifest::new(
            "Residual Source Generation",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();

        let manager = DefaultEditorAssetManager::new();
        manager
            .sync_from_project(ProjectManager::open(&root).unwrap())
            .unwrap();
        {
            let mut state = manager
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let asset_index = state.asset_index.clone();
            *state = Default::default();
            state.asset_index = asset_index;
            assert!(state.asset_index.is_some());
        }

        assert!(EditorAssetManager::deactivate_runtime_project(&manager));
        let state = manager
            .state
            .read()
            .expect("editor asset state lock poisoned");
        assert!(state.asset_index.is_none());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_project_deactivation_recovers_from_a_poisoned_state_lock() {
        let manager = DefaultEditorAssetManager::new();
        let state = Arc::clone(&manager.state);
        let poisoner = std::thread::spawn(move || {
            let _guard = state.write().expect("state write lock");
            panic!("poison the editor asset state lock");
        });
        assert!(poisoner.join().is_err());

        assert!(!EditorAssetManager::deactivate_runtime_project(&manager));
    }

    #[test]
    fn runtime_project_deactivation_recovers_from_a_poisoned_source_sync_gate() {
        let manager = DefaultEditorAssetManager::new();
        let gate = Arc::clone(&manager.source_sync_gate);
        let poisoner = std::thread::spawn(move || {
            let _guard = gate.lock().expect("source sync gate");
            panic!("poison the source sync gate");
        });
        assert!(poisoner.join().is_err());

        assert!(!EditorAssetManager::deactivate_runtime_project(&manager));
    }

    #[test]
    fn runtime_project_deactivation_recovers_from_a_poisoned_publish_gate() {
        let manager = DefaultEditorAssetManager::new();
        {
            let mut state = manager.state.write().expect("editor asset state lock");
            state.project_root = Some(std::path::PathBuf::from("E:/projects/retired"));
        }
        let gate = Arc::clone(&manager.publish_gate);
        let poisoner = std::thread::spawn(move || {
            let _guard = gate.lock().expect("publish gate");
            panic!("poison the publish gate");
        });
        assert!(poisoner.join().is_err());

        assert!(EditorAssetManager::deactivate_runtime_project(&manager));
    }

    #[test]
    #[should_panic(expected = "editor asset source-sync epoch exhausted")]
    fn runtime_project_deactivation_never_reuses_an_exhausted_source_sync_epoch() {
        let manager = DefaultEditorAssetManager::new();
        manager.source_sync_epoch.store(u64::MAX, Ordering::Release);

        let _ = EditorAssetManager::deactivate_runtime_project(&manager);
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
