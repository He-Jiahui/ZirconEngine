use std::path::PathBuf;

use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::core::resource::ResourceMutationBatch;
use crate::core::CoreError;

use super::super::resource_sync::project_locators;
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Retires the active project generation and returns its committed root.
    ///
    /// The generation write lock prevents watcher callbacks from observing a partially retired
    /// project while the watcher activation, resource registry, source-path index, and project
    /// snapshot transition together. It also spans `Removed` publication so a later generation's
    /// `Added` publication cannot overtake the close event.
    pub(crate) fn close_project(&self) -> Result<Option<PathBuf>, CoreError> {
        let generation = self.project_generation_write();
        let (root, removed_changes, retired_watchers) = {
            let mut project = self.project_write();
            let Some(active_project) = project.as_ref() else {
                return Ok(None);
            };
            self.begin_project_preparation();
            let root = active_project.paths().root().to_path_buf();
            let locators = project_locators(active_project);
            let removed_changes = locators
                .iter()
                .cloned()
                .map(|uri| AssetChange::new(AssetChangeKind::Removed, uri, None))
                .collect();

            let mut batch = ResourceMutationBatch::new();
            for locator in locators {
                batch = batch.remove(locator);
            }
            let mut retired_watchers = None;
            self.commit_resource_batch_after_dependencies(batch, || {
                retired_watchers = Some(self.deactivate_project_watchers());
                self.clear_project_source_paths();
                self.clear_transaction_watch_echoes();
                *project = None;
                drop(project);
                Ok(())
            })?;

            (
                root,
                removed_changes,
                retired_watchers.expect("successful project retirement deactivates its watchers"),
            )
        };

        self.publish_project_generation(generation, removed_changes);
        drop(retired_watchers);
        Ok(Some(root))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, TryLockError};
    use std::thread;
    use std::time::{Duration, Instant};

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
        let preparation_epoch = manager.current_project_preparation_epoch();

        assert_eq!(AssetManager::close_project(&manager).unwrap(), None);
        assert_eq!(
            manager.current_project_preparation_epoch(),
            preparation_epoch,
            "a no-op close must not supersede an in-flight project preparation"
        );
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

    #[test]
    fn close_project_publishes_removed_before_the_next_generation_added() {
        let (first_root, first_uri) = create_project(
            "asset_manager_close_order_first",
            "Close Order First",
            "models/first.obj",
        );
        let (second_root, second_uri) = create_project(
            "asset_manager_close_order_second",
            "Close Order Second",
            "models/second.obj",
        );

        let manager = Arc::new(ProjectAssetManager::default());
        manager
            .register_first_wave_plugin_fixture_importers_for_test()
            .unwrap();
        let changes = AssetManager::subscribe_asset_changes(manager.as_ref());
        AssetManager::open_project(manager.as_ref(), first_root.to_string_lossy().as_ref())
            .unwrap();
        while changes.recv_timeout(Duration::from_millis(25)).is_ok() {}

        let subscribers = manager.lock_change_subscribers();
        let close_manager = Arc::clone(&manager);
        let close = thread::spawn(move || AssetManager::close_project(close_manager.as_ref()));

        let close_commit_deadline = Instant::now() + Duration::from_secs(2);
        let mut close_snapshot_retired = false;
        let mut project_state_poisoned = false;
        while Instant::now() < close_commit_deadline {
            match manager.project.try_read() {
                Ok(project) if project.is_none() => {
                    close_snapshot_retired = true;
                    break;
                }
                Ok(_) | Err(TryLockError::WouldBlock) => thread::yield_now(),
                Err(TryLockError::Poisoned(poisoned)) => {
                    drop(poisoned.into_inner());
                    project_state_poisoned = true;
                    break;
                }
            }
        }
        let generation_read_blocked = matches!(
            manager.project_generation_gate.try_read(),
            Err(TryLockError::WouldBlock)
        );
        let generation_write_blocked = matches!(
            manager.project_generation_gate.try_write(),
            Err(TryLockError::WouldBlock)
        );

        let preparation_epoch = manager.current_project_preparation_epoch();
        let open_manager = Arc::clone(&manager);
        let second_root_for_open = second_root.clone();
        let open = thread::spawn(move || {
            AssetManager::open_project(
                open_manager.as_ref(),
                second_root_for_open.to_string_lossy().as_ref(),
            )
        });
        let preparation_deadline = Instant::now() + Duration::from_secs(2);
        while manager.current_project_preparation_epoch() == preparation_epoch
            && Instant::now() < preparation_deadline
        {
            thread::yield_now();
        }
        let next_generation_started =
            manager.current_project_preparation_epoch() != preparation_epoch;

        drop(subscribers);
        let close_result = close.join().unwrap();
        let open_result = open.join().unwrap();

        assert!(
            !project_state_poisoned,
            "project state poisoned during close"
        );
        assert!(
            close_snapshot_retired,
            "close did not retire its project snapshot before publication"
        );
        assert!(
            generation_read_blocked && generation_write_blocked,
            "close publication must retain the project generation write fence"
        );
        assert!(
            next_generation_started,
            "next project generation did not begin preparation"
        );
        assert_eq!(close_result.unwrap(), Some(first_root.clone()));
        open_result.unwrap();

        let event_deadline = Instant::now() + Duration::from_secs(2);
        let mut relevant = Vec::new();
        while relevant.len() < 2 && Instant::now() < event_deadline {
            if let Ok(change) = changes.recv_timeout(Duration::from_millis(25)) {
                if change.uri == first_uri || change.uri == second_uri {
                    relevant.push((change.kind, change.uri));
                }
            }
        }
        assert_eq!(
            relevant,
            vec![
                (AssetChangeKind::Removed, first_uri),
                (AssetChangeKind::Added, second_uri),
            ]
        );

        drop(manager);
        let _ = std::fs::remove_dir_all(first_root);
        let _ = std::fs::remove_dir_all(second_root);
    }

    fn create_project(case: &str, name: &str, asset_path: &str) -> (std::path::PathBuf, AssetUri) {
        let root = unique_temp_project_root(case);
        let paths = ProjectPaths::from_root(&root).unwrap();
        let assets = zircon_runtime_interface::project::RelPath::project_assets();
        paths.ensure_layout(&[assets.clone()]).unwrap();
        ProjectManifest::new(
            name,
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        write_triangle_obj(paths.asset_root(&assets).join(asset_path));
        (
            root,
            AssetUri::parse(&format!("res://{asset_path}")).unwrap(),
        )
    }
}
