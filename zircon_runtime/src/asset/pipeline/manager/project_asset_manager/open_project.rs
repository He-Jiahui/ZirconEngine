use crate::asset::ProjectInfo;
use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::core::CoreError;

use super::super::errors::asset_error;
use super::super::records::build_project_info;
use super::super::resource_sync::{clear_removed_project_resources, project_locators};
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Activates an already parsed project as the authoritative runtime asset project.
    ///
    /// Runtime session construction uses this entry after reading the manifest once for plugin
    /// selection. The path-based [`crate::asset::AssetManager::open_project`] entry delegates here
    /// as well so both startup routes share importer registration, scanning, resource sync, watcher
    /// replacement, and change publication semantics.
    pub(crate) fn open_prepared_project(
        &self,
        mut project: ProjectManager,
    ) -> Result<ProjectInfo, CoreError> {
        let _generation = self.project_generation_write();
        let installed_importers = self.importer_registry_read().clone();
        project
            .register_asset_importers_from_registry(&installed_importers)
            .map_err(asset_error)?;
        project.set_environment_ibl_parallel_executor(self.worker_task_pool.clone());
        let previous_locators = self
            .project_read()
            .as_ref()
            .map(project_locators)
            .unwrap_or_default();
        let prepared_watchers = self.prepare_project_watchers(&project)?;
        let imported = project.scan_and_import().map_err(asset_error)?;
        let prepared_resources = self.prepare_project_resource_sync(&project)?;
        let info = build_project_info(&project);
        let (retired_watchers, watcher_activation) = {
            let mut active_project = self.project_write();
            clear_removed_project_resources(&self.resource_manager(), &previous_locators, &project);
            self.commit_project_resource_sync(prepared_resources);
            *active_project = Some(project);
            self.activate_project_watchers(prepared_watchers)
        };
        self.broadcast(
            imported
                .into_iter()
                .map(|metadata| {
                    AssetChange::new(
                        AssetChangeKind::Added,
                        metadata.primary_locator().clone(),
                        None,
                    )
                })
                .collect(),
        );
        drop(_generation);
        drop(retired_watchers);
        self.drain_project_watcher_events(watcher_activation);
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::project::{ProjectManager, ProjectManifest};
    use crate::asset::{AssetManager, AssetUri};
    use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
    use zircon_runtime_interface::project::RelPath;

    use super::ProjectAssetManager;

    #[test]
    fn project_startup_snapshot_survives_disk_manifest_rewrite_after_activation() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_asset_manager_prepared_project_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(root.join("assets")).unwrap();
        let manifest_path = root.join("zircon-project.toml");
        ProjectManifest::new(
            "Activated Snapshot One",
            AssetUri::parse("res://scenes/one.scene.toml").unwrap(),
            1,
        )
        .save(&manifest_path)
        .unwrap();
        let project = ProjectManager::open(&root).unwrap();
        let manager = ProjectAssetManager::default();
        let opened = AssetManager::open_prepared_project(&manager, project).unwrap();
        assert_eq!(opened.name, "Activated Snapshot One");

        ProjectManifest::new(
            "Activated Snapshot Two",
            AssetUri::parse("res://scenes/two.scene.toml").unwrap(),
            2,
        )
        .save(&manifest_path)
        .unwrap();

        let current = AssetManager::current_project_snapshot(&manager).unwrap();

        assert_eq!(
            current.manifest().default_scene.to_string(),
            "res://scenes/one.scene.toml"
        );
        drop(current);
        drop(manager);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_project_uses_manager_owned_io_pool_for_environment_ibl_staging() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_asset_manager_ibl_executor_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(root.join("assets")).unwrap();
        ProjectManifest::new(
            "IBL Executor Injection",
            AssetUri::parse("res://scenes/ibl.scene.toml").unwrap(),
            1,
        )
        .save(root.join("zircon-project.toml"))
        .unwrap();

        let task_pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1));
        let manager = ProjectAssetManager::new(task_pool.clone());
        let project = ProjectManager::open(&root).unwrap();
        AssetManager::open_prepared_project(&manager, project).unwrap();

        let project = AssetManager::current_project_snapshot(&manager).unwrap();
        let injected = project
            .environment_ibl_parallel_executor_for_test()
            .expect("prepared project should retain the manager runtime IO pool");
        assert!(injected.shares_execution_owner_with(&task_pool));

        drop(project);
        drop(manager);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn failed_scan_stops_pending_watchers_without_publishing_a_generation() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_asset_manager_failed_scan_watchers_{}_{}",
            std::process::id(),
            unique
        ));
        let mut manifest = ProjectManifest::new(
            "Failed Scan Watchers",
            AssetUri::parse("res://data/collision.json").unwrap(),
            1,
        );
        manifest.asset_roots = vec![
            RelPath::parse("game-assets").unwrap(),
            RelPath::parse("shared-assets").unwrap(),
        ];
        manifest.save(root.join("zircon-project.toml")).unwrap();
        let first = root.join("game-assets/data/collision.json");
        let second = root.join("shared-assets/data/collision.json");
        fs::create_dir_all(first.parent().unwrap()).unwrap();
        fs::create_dir_all(second.parent().unwrap()).unwrap();
        fs::write(first, "{}").unwrap();
        fs::write(second, "{}").unwrap();

        let manager = ProjectAssetManager::default();
        let project = ProjectManager::open(&root).unwrap();

        assert!(AssetManager::open_prepared_project(&manager, project).is_err());
        assert!(manager.project_read().is_none());
        assert!(
            manager
                .watcher_activation
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .is_none()
        );
        assert!(
            manager
                .watchers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .is_empty()
        );

        drop(manager);
        let _ = fs::remove_dir_all(root);
    }
}
