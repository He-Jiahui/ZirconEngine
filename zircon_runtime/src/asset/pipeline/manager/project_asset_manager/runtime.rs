use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::resource_sync::register_project_resource;
use super::ProjectAssetManager;
use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetWatchError, AssetWatcher};
use crate::core::resource::ResourceState;

impl ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) fn project_read(
        &self,
    ) -> RwLockReadGuard<'_, Option<ProjectManager>> {
        self.project.read().expect("asset project lock poisoned")
    }

    pub(in crate::asset::pipeline::manager) fn project_write(
        &self,
    ) -> RwLockWriteGuard<'_, Option<ProjectManager>> {
        self.project.write().expect("asset project lock poisoned")
    }

    pub(in crate::asset::pipeline::manager) fn broadcast(&self, changes: Vec<AssetChange>) {
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

    pub(in crate::asset::pipeline::manager) fn broadcast_watch_error(
        &self,
        error: AssetWatchError,
    ) {
        let mut subscribers = self
            .watch_error_subscribers
            .lock()
            .expect("asset watch error subscribers lock poisoned");
        subscribers.retain(|sender| sender.send(error.clone()).is_ok());
    }

    pub(in crate::asset::pipeline::manager) fn restart_watcher(&self) -> Result<(), CoreError> {
        let assets_root = {
            let project = self.project_read();
            let project = project
                .as_ref()
                .ok_or_else(|| asset_error_message("no project is currently open"))?;
            project.paths().assets_root().to_path_buf()
        };
        let manager = self.clone();
        let error_manager = self.clone();
        let watcher = AssetWatcher::spawn(
            assets_root,
            move |changes| {
                manager.process_watch_changes(changes);
            },
            move |error| {
                error_manager.broadcast_watch_error(error);
            },
        )
        .map_err(asset_error)?;
        *self.watcher.lock().expect("asset watcher lock poisoned") = Some(watcher);
        Ok(())
    }

    pub(in crate::asset::pipeline::manager) fn sync_project_resources(
        &self,
        project: &ProjectManager,
    ) -> Result<(), CoreError> {
        for metadata in project.registry().values() {
            if metadata.state == ResourceState::Error || metadata.artifact_locator().is_none() {
                self.resource_manager.register_record(metadata.clone());
                continue;
            }
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
            if let Err(error) = project.scan_and_import() {
                self.broadcast_watch_error(AssetWatchError::from_message(
                    project.paths().assets_root().to_path_buf(),
                    error.to_string(),
                ));
                return;
            }
            if let Err(error) = self.sync_project_resources(project) {
                self.broadcast_watch_error(AssetWatchError::from_message(
                    project.paths().assets_root().to_path_buf(),
                    error.to_string(),
                ));
                return;
            }
        }

        self.broadcast(
            changes
                .into_iter()
                .map(|change| AssetChange::new(change.kind, change.uri, change.previous_uri))
                .collect(),
        );
    }
}
