use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::ProjectInfo;
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
        let installed_importers = self.importer_registry_read().clone();
        project
            .register_asset_importers_from_registry(&installed_importers)
            .map_err(asset_error)?;
        let previous_locators = self
            .project_read()
            .as_ref()
            .map(project_locators)
            .unwrap_or_default();
        let imported = project.scan_and_import().map_err(asset_error)?;
        clear_removed_project_resources(&self.resource_manager(), &previous_locators, &project);
        self.sync_project_resources(&project)?;
        let info = build_project_info(&project);
        *self.project_write() = Some(project);
        self.restart_watcher()?;
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
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::project::{ProjectManager, ProjectManifest};
    use crate::asset::{AssetManager, AssetUri};

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
}
