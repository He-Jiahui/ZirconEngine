use std::sync::Arc;
use std::sync::atomic::Ordering;

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
            if state.project.is_none() && state.catalog_generation.assets.is_empty() {
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::Ordering;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::AssetUri;
    use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};

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
