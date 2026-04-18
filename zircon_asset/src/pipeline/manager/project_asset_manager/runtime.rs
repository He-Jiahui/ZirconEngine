use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use zircon_core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::resource_sync::register_project_resource;
use super::ProjectAssetManager;
use crate::{AssetChange, AssetWatcher, ProjectManager};

impl ProjectAssetManager {
    pub(in crate::pipeline::manager) fn project_read(
        &self,
    ) -> RwLockReadGuard<'_, Option<ProjectManager>> {
        self.project.read().expect("asset project lock poisoned")
    }

    pub(in crate::pipeline::manager) fn project_write(
        &self,
    ) -> RwLockWriteGuard<'_, Option<ProjectManager>> {
        self.project.write().expect("asset project lock poisoned")
    }

    pub(in crate::pipeline::manager) fn broadcast(&self, changes: Vec<AssetChange>) {
        if changes.is_empty() {
            return;
        }

        let mut subscribers = self
            .change_subscribers
            .lock()
            .expect("asset subscribers lock poisoned");
        subscribers.retain(|sender| {
            changes
                .iter()
                .all(|change| sender.send(change.clone()).is_ok())
        });
    }

    pub(in crate::pipeline::manager) fn restart_watcher(&self) -> Result<(), CoreError> {
        let assets_root = {
            let project = self.project_read();
            let project = project
                .as_ref()
                .ok_or_else(|| asset_error_message("no project is currently open"))?;
            project.paths().assets_root().to_path_buf()
        };
        let manager = self.clone();
        let watcher = AssetWatcher::spawn(assets_root, move |changes| {
            manager.process_watch_changes(changes);
        })
        .map_err(asset_error)?;
        *self.watcher.lock().expect("asset watcher lock poisoned") = Some(watcher);
        Ok(())
    }

    pub(in crate::pipeline::manager) fn sync_project_resources(
        &self,
        project: &ProjectManager,
    ) -> Result<(), CoreError> {
        for metadata in project.registry().values() {
            let imported = project
                .load_artifact_by_id(metadata.id())
                .map_err(asset_error)?;
            register_project_resource(&self.resource_manager, metadata.clone(), imported);
        }
        Ok(())
    }

    pub(super) fn process_watch_changes(&self, changes: Vec<AssetChange>) {
        if changes.is_empty() {
            return;
        }

        {
            let mut project = self.project_write();
            let Some(project) = project.as_mut() else {
                return;
            };
            if project.scan_and_import().is_err() || self.sync_project_resources(project).is_err() {
                return;
            }
            let _ = self.editor_asset_manager.sync_from_project(project.clone());
        }

        self.broadcast(
            changes
                .into_iter()
                .map(|change| AssetChange::new(change.kind, change.uri, change.previous_uri))
                .collect(),
        );
    }
}
