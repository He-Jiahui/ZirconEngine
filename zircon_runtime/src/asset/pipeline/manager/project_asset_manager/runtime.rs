use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use crate::asset::{AssetImporterRegistry, ProjectManager};
use crate::core::framework::channel::ChannelSender;
use crate::core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::resource_sync::register_project_resource;
use super::ProjectAssetManager;
use crate::asset::watch::{AssetChange, AssetWatchError, AssetWatcher};
use crate::core::resource::ResourceState;

impl ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) fn project_read(
        &self,
    ) -> RwLockReadGuard<'_, Option<ProjectManager>> {
        self.project
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn project_write(
        &self,
    ) -> RwLockWriteGuard<'_, Option<ProjectManager>> {
        self.project
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn importer_registry_read(
        &self,
    ) -> RwLockReadGuard<'_, AssetImporterRegistry> {
        self.asset_importers
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn importer_registry_write(
        &self,
    ) -> RwLockWriteGuard<'_, AssetImporterRegistry> {
        self.asset_importers
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn lock_change_subscribers(
        &self,
    ) -> MutexGuard<'_, Vec<ChannelSender<AssetChange>>> {
        self.change_subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn lock_watch_error_subscribers(
        &self,
    ) -> MutexGuard<'_, Vec<ChannelSender<AssetWatchError>>> {
        self.watch_error_subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_watchers(&self) -> MutexGuard<'_, Vec<AssetWatcher>> {
        self.watchers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn broadcast(&self, changes: Vec<AssetChange>) {
        if changes.is_empty() {
            return;
        }

        let mut subscribers = self.lock_change_subscribers();
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
        let mut subscribers = self.lock_watch_error_subscribers();
        subscribers.retain(|sender| sender.send(error.clone()).is_ok());
    }

    pub(in crate::asset::pipeline::manager) fn restart_watcher(&self) -> Result<(), CoreError> {
        let asset_roots = {
            let project = self.project_read();
            let project = project
                .as_ref()
                .ok_or_else(|| asset_error_message("no project is currently open"))?;
            project.project_asset_roots().to_vec()
        };
        let mut watchers = Vec::with_capacity(asset_roots.len());
        for asset_root in asset_roots {
            let manager = self.clone();
            let error_manager = self.clone();
            watchers.push(
                AssetWatcher::spawn(
                    asset_root,
                    move |changes| {
                        manager.process_watch_changes(changes);
                    },
                    move |error| {
                        error_manager.broadcast_watch_error(error);
                    },
                )
                .map_err(asset_error)?,
            );
        }
        *self.lock_watchers() = watchers;
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
            let watch_error_root = match project.primary_project_asset_root() {
                Ok(root) => root.to_path_buf(),
                Err(_) => project.paths().root().to_path_buf(),
            };
            if let Err(error) = project.scan_and_import_watch_changes(&changes) {
                self.broadcast_watch_error(AssetWatchError::from_message(
                    watch_error_root.clone(),
                    error.to_string(),
                ));
                return;
            }
            if let Err(error) = self.sync_project_resources(project) {
                self.broadcast_watch_error(AssetWatchError::from_message(
                    watch_error_root,
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

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    #[test]
    fn project_asset_manager_runtime_accessors_recover_poisoned_locks() {
        let manager = ProjectAssetManager::new(1);

        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.project.write().unwrap();
            panic!("poison project lock");
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.asset_importers.write().unwrap();
            panic!("poison importer registry lock");
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.change_subscribers.lock().unwrap();
            panic!("poison change subscribers lock");
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.watch_error_subscribers.lock().unwrap();
            panic!("poison watch error subscribers lock");
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.watchers.lock().unwrap();
            panic!("poison watchers lock");
        }))
        .is_err());

        assert!(manager.project_read().is_none());
        assert!(manager.importer_registry_read().importers().is_empty());
        assert!(manager.lock_change_subscribers().is_empty());
        assert!(manager.lock_watch_error_subscribers().is_empty());
        assert!(manager.lock_watchers().is_empty());
    }
}
