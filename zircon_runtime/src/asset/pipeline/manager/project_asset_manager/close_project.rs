use std::path::PathBuf;

use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::core::CoreError;

use super::super::resource_sync::project_locators;
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Retires the active project generation and returns its committed root.
    ///
    /// The generation write lock prevents watcher callbacks from observing a partially retired
    /// project while the watcher activation, resource registry, source-path index, and project
    /// snapshot transition together.
    pub(crate) fn close_project(&self) -> Result<Option<PathBuf>, CoreError> {
        let (root, removed_changes, retired_watchers) = {
            let _generation = self.project_generation_write();
            let mut project = self.project_write();
            let Some(active_project) = project.as_ref() else {
                return Ok(None);
            };
            let root = active_project.paths().root().to_path_buf();
            let locators = project_locators(active_project);
            let removed_changes = locators
                .iter()
                .cloned()
                .map(|uri| AssetChange::new(AssetChangeKind::Removed, uri, None))
                .collect();

            let retired_watchers = self.deactivate_project_watchers();
            let resource_manager = self.resource_manager();
            for locator in locators {
                let _ = resource_manager.remove_by_locator(&locator);
            }
            self.clear_project_source_paths();
            *project = None;

            (root, removed_changes, retired_watchers)
        };

        drop(retired_watchers);
        self.broadcast(removed_changes);
        Ok(Some(root))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::asset::project::{ProjectManifest, ProjectPaths};
    use crate::asset::tests::project::unique_temp_project_root;
    use crate::asset::tests::support::write_triangle_obj;
    use crate::asset::watch::AssetChangeKind;
    use crate::asset::{AssetManager, AssetUri};

    use super::ProjectAssetManager;

    #[test]
    fn close_project_without_an_active_project_is_a_noop() {
        let manager = ProjectAssetManager::default();
        let changes = AssetManager::subscribe_asset_changes(&manager);

        assert_eq!(AssetManager::close_project(&manager).unwrap(), None);
        assert!(AssetManager::current_project_snapshot(&manager).is_none());
        assert!(changes.try_recv().is_err());
    }

    #[test]
    fn close_project_retires_project_resources_and_publishes_removed_changes() {
        let root = unique_temp_project_root("asset_manager_close_project");
        let paths = ProjectPaths::from_root(&root).unwrap();
        let assets = zircon_runtime_interface::project::RelPath::project_assets();
        paths.ensure_layout(&[assets.clone()]).unwrap();
        ProjectManifest::new(
            "Close Project",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        write_triangle_obj(paths.asset_root(&assets).join("models/triangle.obj"));

        let manager = ProjectAssetManager::default();
        manager
            .register_first_wave_plugin_fixture_importers_for_test()
            .unwrap();
        let changes = AssetManager::subscribe_asset_changes(&manager);
        AssetManager::open_project(&manager, root.to_string_lossy().as_ref()).unwrap();
        while changes.recv_timeout(Duration::from_millis(25)).is_ok() {}

        let uri = AssetUri::parse("res://models/triangle.obj").unwrap();
        assert!(manager.resolve_asset_id(&uri).is_some());

        assert_eq!(
            AssetManager::close_project(&manager).unwrap(),
            Some(root.clone())
        );
        assert!(AssetManager::current_project_snapshot(&manager).is_none());
        assert!(manager.resolve_asset_id(&uri).is_none());
        assert!(manager.indexed_project_source_path(&uri).is_none());

        let removed = changes.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(removed.kind, AssetChangeKind::Removed);
        assert_eq!(removed.uri, uri);

        drop(manager);
        let _ = std::fs::remove_dir_all(root);
    }
}
