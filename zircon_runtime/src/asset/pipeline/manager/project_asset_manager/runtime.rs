use std::collections::hash_map::Entry;
use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use crate::asset::{AssetImporterRegistry, AssetUri, ImportedAsset, ProjectManager};
use crate::core::framework::channel::ChannelSender;
use crate::core::CoreError;

use super::super::errors::asset_error;
use super::super::resource_sync::register_project_resource;
use super::project_asset_manager::{
    ProjectSourcePathIndex, ProjectWatcherActivation, ProjectWatcherActivationState,
    ProjectWatcherLifecycle,
};
use super::ProjectAssetManager;
use crate::asset::watch::{AssetChange, AssetWatchError, AssetWatcher};
use crate::core::resource::{ResourceRecord, ResourceScheme, ResourceState};

pub(in crate::asset::pipeline::manager) struct PreparedProjectResourceSync {
    source_paths: ProjectSourcePathIndex,
    resources: Vec<PreparedProjectResource>,
}

pub(in crate::asset::pipeline::manager) struct PreparedTargetedProjectResourceSync {
    source_uri: AssetUri,
    source_path: std::path::PathBuf,
    removed_locators: Vec<AssetUri>,
    resources: Vec<PreparedProjectResource>,
    record_updates: Vec<ResourceRecord>,
}

enum PreparedProjectResource {
    Record(ResourceRecord),
    Ready(ResourceRecord, ImportedAsset),
}

pub(in crate::asset::pipeline::manager) struct PreparedProjectWatchers {
    watchers: Vec<AssetWatcher>,
    activation: std::sync::Arc<ProjectWatcherActivation>,
}

impl ProjectWatcherActivation {
    fn lock_state(&self) -> MutexGuard<'_, ProjectWatcherActivationState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn dispatch_changes(&self, manager: &ProjectAssetManager, changes: Vec<AssetChange>) {
        let changes = self.active_changes_or_queue(changes);
        if let Some(changes) = changes {
            let _generation = manager.project_generation_read();
            if self.is_active() {
                manager.process_watch_changes_in_generation(changes);
            }
        }
    }

    fn active_changes_or_queue(&self, changes: Vec<AssetChange>) -> Option<Vec<AssetChange>> {
        let mut state = self.lock_state();
        match state.lifecycle {
            ProjectWatcherLifecycle::Pending | ProjectWatcherLifecycle::Draining => {
                state.changes.extend(changes);
                None
            }
            ProjectWatcherLifecycle::Active => Some(changes),
            ProjectWatcherLifecycle::Retired => None,
        }
    }

    fn dispatch_error(&self, manager: &ProjectAssetManager, error: AssetWatchError) {
        let dispatch = {
            let mut state = self.lock_state();
            match state.lifecycle {
                ProjectWatcherLifecycle::Pending | ProjectWatcherLifecycle::Draining => {
                    state.errors.push(error.clone());
                    false
                }
                ProjectWatcherLifecycle::Active => true,
                ProjectWatcherLifecycle::Retired => false,
            }
        };
        if dispatch {
            let _generation = manager.project_generation_read();
            if self.is_active() {
                manager.broadcast_watch_error(error);
            }
        }
    }

    fn is_active(&self) -> bool {
        self.lock_state().lifecycle == ProjectWatcherLifecycle::Active
    }

    fn begin_draining(&self) {
        let mut state = self.lock_state();
        if state.lifecycle == ProjectWatcherLifecycle::Pending {
            state.lifecycle = ProjectWatcherLifecycle::Draining;
        }
    }

    fn drain(&self, manager: &ProjectAssetManager) {
        loop {
            let _generation = manager.project_generation_read();
            let pending = {
                let mut state = self.lock_state();
                if state.lifecycle != ProjectWatcherLifecycle::Draining {
                    return;
                }
                if state.changes.is_empty() && state.errors.is_empty() {
                    state.lifecycle = ProjectWatcherLifecycle::Active;
                    return;
                }
                (
                    std::mem::take(&mut state.changes),
                    std::mem::take(&mut state.errors),
                )
            };
            if !pending.0.is_empty() {
                manager.process_watch_changes_in_generation(pending.0);
            }
            for error in pending.1 {
                manager.broadcast_watch_error(error);
            }
        }
    }

    fn retire(&self) {
        let mut state = self.lock_state();
        state.lifecycle = ProjectWatcherLifecycle::Retired;
        state.changes.clear();
        state.errors.clear();
    }
}

impl ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) fn project_generation_read(
        &self,
    ) -> RwLockReadGuard<'_, ()> {
        self.project_generation_gate
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn project_generation_write(
        &self,
    ) -> RwLockWriteGuard<'_, ()> {
        self.project_generation_gate
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

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

    pub(in crate::asset::pipeline::manager) fn project_source_paths_read(
        &self,
    ) -> RwLockReadGuard<'_, ProjectSourcePathIndex> {
        self.project_source_paths
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn project_source_paths_write(&self) -> RwLockWriteGuard<'_, ProjectSourcePathIndex> {
        self.project_source_paths
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::asset::pipeline::manager) fn indexed_project_source_path(
        &self,
        uri: &AssetUri,
    ) -> Option<std::path::PathBuf> {
        self.project_source_paths_read()
            .get(&uri.scheme())
            .and_then(
                |paths: &std::collections::HashMap<String, std::path::PathBuf>| {
                    paths.get(uri.path())
                },
            )
            .cloned()
    }

    fn build_project_source_paths(
        project: &ProjectManager,
    ) -> Result<ProjectSourcePathIndex, CoreError> {
        let mut source_paths = ProjectSourcePathIndex::new();
        for record in project.registry().values() {
            let locator = record.primary_locator();
            if !matches!(
                locator.scheme(),
                ResourceScheme::Res | ResourceScheme::Package
            ) {
                continue;
            }
            let scheme_paths = source_paths.entry(locator.scheme()).or_default();
            if let Entry::Vacant(entry) = scheme_paths.entry(locator.path().to_string()) {
                entry.insert(project.source_path_for_uri(locator).map_err(asset_error)?);
            }
        }
        Ok(source_paths)
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

    fn lock_watcher_activation(
        &self,
    ) -> MutexGuard<'_, Option<std::sync::Arc<ProjectWatcherActivation>>> {
        self.watcher_activation
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

    pub(in crate::asset::pipeline::manager) fn prepare_project_watchers(
        &self,
        project: &ProjectManager,
    ) -> Result<PreparedProjectWatchers, CoreError> {
        let activation = std::sync::Arc::new(ProjectWatcherActivation {
            state: std::sync::Mutex::new(ProjectWatcherActivationState {
                lifecycle: ProjectWatcherLifecycle::Pending,
                changes: Vec::new(),
                errors: Vec::new(),
            }),
        });
        let asset_roots = project.project_asset_roots().to_vec();
        let mut watchers = Vec::with_capacity(asset_roots.len());
        for asset_root in asset_roots {
            let manager = self.clone();
            let error_manager = self.clone();
            let change_activation = activation.clone();
            let error_activation = activation.clone();
            watchers.push(
                AssetWatcher::spawn(
                    asset_root,
                    move |changes| {
                        change_activation.dispatch_changes(&manager, changes);
                    },
                    move |error| {
                        error_activation.dispatch_error(&error_manager, error);
                    },
                )
                .map_err(asset_error)?,
            );
        }
        Ok(PreparedProjectWatchers {
            watchers,
            activation,
        })
    }

    pub(in crate::asset::pipeline::manager) fn activate_project_watchers(
        &self,
        prepared: PreparedProjectWatchers,
    ) -> (Vec<AssetWatcher>, std::sync::Arc<ProjectWatcherActivation>) {
        let mut active: MutexGuard<'_, Option<std::sync::Arc<ProjectWatcherActivation>>> =
            self.lock_watcher_activation();
        if let Some(previous) = active.take() {
            previous.retire();
        }
        let retired_watchers = std::mem::replace(&mut *self.lock_watchers(), prepared.watchers);
        prepared.activation.begin_draining();
        *active = Some(prepared.activation.clone());
        (retired_watchers, prepared.activation)
    }

    pub(in crate::asset::pipeline::manager) fn deactivate_project_watchers(
        &self,
    ) -> Vec<AssetWatcher> {
        if let Some(activation) = self.lock_watcher_activation().take() {
            activation.retire();
        }
        std::mem::take(&mut *self.lock_watchers())
    }

    pub(in crate::asset::pipeline::manager) fn drain_project_watcher_events(
        &self,
        activation: std::sync::Arc<ProjectWatcherActivation>,
    ) {
        activation.drain(self);
    }

    /// Stops project asset watchers before the owning runtime releases its services.
    ///
    /// Watch callbacks retain manager clones, so waiting for the manager's final drop would leave
    /// the watcher join handles in a reference cycle on Windows.
    pub fn shutdown_project_watchers(&self) {
        let retired_watchers = self.deactivate_project_watchers();
        drop(retired_watchers);
    }

    pub(in crate::asset::pipeline::manager) fn clear_project_source_paths(&self) {
        self.project_source_paths_write().clear();
    }

    pub(in crate::asset::pipeline::manager) fn prepare_project_resource_sync(
        &self,
        project: &ProjectManager,
    ) -> Result<PreparedProjectResourceSync, CoreError> {
        let source_paths = Self::build_project_source_paths(project)?;
        let mut resources = Vec::with_capacity(project.registry().values().count());
        for metadata in project.registry().values() {
            if metadata.state == ResourceState::Error || metadata.artifact_locator().is_none() {
                resources.push(PreparedProjectResource::Record(metadata.clone()));
                continue;
            }
            let imported = project
                .load_artifact_by_id(metadata.id())
                .map_err(asset_error)?;
            resources.push(PreparedProjectResource::Ready(metadata.clone(), imported));
        }
        Ok(PreparedProjectResourceSync {
            source_paths,
            resources,
        })
    }

    pub(in crate::asset::pipeline::manager) fn commit_project_resource_sync(
        &self,
        prepared: PreparedProjectResourceSync,
    ) {
        for resource in prepared.resources {
            match resource {
                PreparedProjectResource::Record(metadata) => {
                    self.resource_manager.register_record(metadata);
                }
                PreparedProjectResource::Ready(metadata, imported) => {
                    register_project_resource(&self.resource_manager, metadata, imported);
                }
            }
        }
        *self.project_source_paths_write() = prepared.source_paths;
    }

    pub(in crate::asset::pipeline::manager) fn prepare_targeted_project_resource_sync(
        &self,
        project: &ProjectManager,
        source_uri: &AssetUri,
        source_path: std::path::PathBuf,
        previous_source_records: &[ResourceRecord],
        imported: &[ResourceRecord],
        affected: &[ResourceRecord],
        ready_payloads: Vec<(ResourceRecord, ImportedAsset)>,
    ) -> PreparedTargetedProjectResourceSync {
        let current_source_records = project.source_resource_records(source_uri);
        let current_locators = current_source_records
            .iter()
            .map(|record| record.primary_locator().clone())
            .collect::<std::collections::HashSet<_>>();
        let removed_locators = previous_source_records
            .iter()
            .map(|record| record.primary_locator().clone())
            .filter(|locator| !current_locators.contains(locator))
            .collect();
        let imported_ids = imported
            .iter()
            .map(ResourceRecord::id)
            .collect::<std::collections::HashSet<_>>();
        let resources = ready_payloads
            .into_iter()
            .map(|(metadata, payload)| PreparedProjectResource::Ready(metadata, payload))
            .collect();
        let record_updates = affected
            .iter()
            .filter(|record| !imported_ids.contains(&record.id()))
            .cloned()
            .collect();
        PreparedTargetedProjectResourceSync {
            source_uri: AssetUri::new(source_uri.scheme(), source_uri.path().to_string(), None)
                .expect("a parsed source URI remains valid when its label is removed"),
            source_path,
            removed_locators,
            resources,
            record_updates,
        }
    }

    pub(in crate::asset::pipeline::manager) fn commit_targeted_project_resource_sync(
        &self,
        prepared: PreparedTargetedProjectResourceSync,
    ) {
        for locator in prepared.removed_locators {
            let _ = self.resource_manager.remove_by_locator(&locator);
        }
        for resource in prepared.resources {
            match resource {
                PreparedProjectResource::Record(metadata) => {
                    self.resource_manager.register_record(metadata);
                }
                PreparedProjectResource::Ready(metadata, payload) => {
                    register_project_resource(&self.resource_manager, metadata, payload);
                }
            }
        }
        for record in prepared.record_updates {
            self.resource_manager.register_record(record);
        }
        self.project_source_paths_write()
            .entry(prepared.source_uri.scheme())
            .or_default()
            .insert(prepared.source_uri.path().to_string(), prepared.source_path);
    }

    fn process_watch_changes_in_generation(&self, changes: Vec<AssetChange>) {
        if changes.is_empty() {
            return;
        }

        {
            let mut project = self.project_write();
            let Some(active_project) = project.as_mut() else {
                return;
            };
            let watch_error_root = match active_project.primary_project_asset_root() {
                Ok(root) => root.to_path_buf(),
                Err(_) => active_project.paths().root().to_path_buf(),
            };
            let previous_locators = super::super::resource_sync::project_locators(active_project);
            let mut candidate = active_project.clone();
            if let Err(error) = candidate.scan_and_import_watch_changes(&changes) {
                self.broadcast_watch_error(AssetWatchError::from_message(
                    watch_error_root.clone(),
                    error.to_string(),
                ));
                return;
            }
            let prepared = match self.prepare_project_resource_sync(&candidate) {
                Ok(prepared) => prepared,
                Err(error) => {
                    self.broadcast_watch_error(AssetWatchError::from_message(
                        watch_error_root,
                        error.to_string(),
                    ));
                    return;
                }
            };
            super::super::resource_sync::clear_removed_project_resources(
                &self.resource_manager,
                &previous_locators,
                &candidate,
            );
            self.commit_project_resource_sync(prepared);
            *active_project = candidate;
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

    use crate::asset::watch::AssetChangeKind;

    use super::*;

    fn watcher_activation(lifecycle: ProjectWatcherLifecycle) -> ProjectWatcherActivation {
        ProjectWatcherActivation {
            state: std::sync::Mutex::new(ProjectWatcherActivationState {
                lifecycle,
                changes: Vec::new(),
                errors: Vec::new(),
            }),
        }
    }

    fn watcher_change(path: &str) -> AssetChange {
        AssetChange::new(
            AssetChangeKind::Modified,
            AssetUri::parse(path).unwrap(),
            None,
        )
    }

    #[test]
    fn watcher_activation_queues_pending_and_draining_events_in_arrival_order() {
        let activation = watcher_activation(ProjectWatcherLifecycle::Pending);

        assert!(activation
            .active_changes_or_queue(vec![watcher_change("res://first.json")])
            .is_none());
        activation.begin_draining();
        assert!(activation
            .active_changes_or_queue(vec![watcher_change("res://second.json")])
            .is_none());

        let state = activation.lock_state();
        assert_eq!(state.lifecycle, ProjectWatcherLifecycle::Draining);
        assert_eq!(
            state
                .changes
                .iter()
                .map(|change| change.uri.to_string())
                .collect::<Vec<_>>(),
            vec!["res://first.json", "res://second.json"]
        );
    }

    #[test]
    fn watcher_activation_rechecks_retirement_after_initial_active_admission() {
        let activation = watcher_activation(ProjectWatcherLifecycle::Active);

        let admitted =
            activation.active_changes_or_queue(vec![watcher_change("res://retired.json")]);
        assert!(admitted.is_some());
        activation.retire();

        assert!(!activation.is_active());
    }

    #[test]
    fn project_asset_manager_runtime_accessors_recover_poisoned_locks() {
        let manager = ProjectAssetManager::default();

        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.project_generation_gate.write().unwrap();
            panic!("poison project generation gate");
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.project.write().unwrap();
            panic!("poison project lock");
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.project_source_paths.write().unwrap();
            panic!("poison project source paths lock");
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
            let _guard = manager.watcher_activation.lock().unwrap();
            panic!("poison watcher activation lock");
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.watchers.lock().unwrap();
            panic!("poison watchers lock");
        }))
        .is_err());

        drop(manager.project_generation_read());
        assert!(manager.project_read().is_none());
        assert!(manager.project_source_paths_read().is_empty());
        assert!(manager.importer_registry_read().importers().is_empty());
        assert!(manager.lock_change_subscribers().is_empty());
        assert!(manager.lock_watch_error_subscribers().is_empty());
        assert!(manager.lock_watcher_activation().is_none());
        assert!(manager.lock_watchers().is_empty());
    }
}
