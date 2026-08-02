use std::collections::hash_map::DefaultHasher;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;
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
use crate::asset::watch::{AssetChange, AssetWatchBatch, AssetWatchError, AssetWatcher};
use crate::core::resource::{ResourceRecord, ResourceScheme};

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

struct PreparedIncrementalProjectResourceSync {
    removed_locators: Vec<AssetUri>,
    source_path_removals: Vec<AssetUri>,
    source_path_updates: Vec<(AssetUri, std::path::PathBuf)>,
    record_updates: Vec<ResourceRecord>,
}

enum PreparedWatchProjectResourceSync {
    Reconciliation(PreparedProjectResourceSync),
    Incremental(PreparedIncrementalProjectResourceSync),
}

enum PreparedProjectResource {
    Record(ResourceRecord),
    Ready(ResourceRecord, ImportedAsset),
}

pub(in crate::asset::pipeline::manager) struct PreparedProjectWatchers {
    watchers: Vec<AssetWatcher>,
    activation: std::sync::Arc<ProjectWatcherActivation>,
}

const WATCH_GENERATION_RETRY_LIMIT: usize = 3;

impl ProjectWatcherActivation {
    pub(super) fn lock_state(&self) -> MutexGuard<'_, ProjectWatcherActivationState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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

    fn retire(&self) {
        let mut state = self.lock_state();
        state.lifecycle = ProjectWatcherLifecycle::Retired;
        state.changes.clear();
        state.coalescible_change_indices.clear();
        state.queued_change_bytes = 0;
        state.requires_reconciliation = false;
        state.diagnostics = Default::default();
        state.errors.clear();
        state.worker_scheduled = false;
    }
}

impl ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) fn begin_project_preparation(&self) -> u64 {
        self.project_preparation_epoch
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1)
    }

    pub(in crate::asset::pipeline::manager) fn is_latest_project_preparation(
        &self,
        epoch: u64,
    ) -> bool {
        self.project_preparation_epoch.load(Ordering::Acquire) == epoch
    }

    pub(in crate::asset::pipeline::manager) fn current_project_preparation_epoch(&self) -> u64 {
        self.project_preparation_epoch.load(Ordering::Acquire)
    }

    pub(in crate::asset::pipeline::manager) fn lock_residency(
        &self,
        id: crate::asset::AssetId,
    ) -> MutexGuard<'_, ()> {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let stripe = hasher.finish() as usize % self.residency_stripes.len();
        self.residency_stripes[stripe]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_watch_refresh(&self) -> MutexGuard<'_, ()> {
        self.watch_refresh_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

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

    pub(in crate::asset::pipeline::manager) fn publish_project_generation(
        &self,
        generation: RwLockWriteGuard<'_, ()>,
        changes: Vec<AssetChange>,
    ) {
        self.broadcast(changes);
        drop(generation);
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
                coalescible_change_indices: Default::default(),
                queued_change_bytes: 0,
                requires_reconciliation: false,
                diagnostics: Default::default(),
                errors: Vec::new(),
                worker_scheduled: false,
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
                    move |batch| {
                        change_activation.enqueue_batch(&manager, batch);
                    },
                    move |error| {
                        error_activation.enqueue_error(&error_manager, error);
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
        activation.activate_dispatch(self);
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
        let resources = project
            .registry()
            .values()
            .cloned()
            .map(PreparedProjectResource::Record)
            .collect();
        Ok(PreparedProjectResourceSync {
            source_paths,
            resources,
        })
    }

    pub(in crate::asset::pipeline::manager) fn commit_project_resource_sync(
        &self,
        prepared: PreparedProjectResourceSync,
    ) {
        let mut lazy_records = Vec::new();
        let mut ready_resources = Vec::new();
        for resource in prepared.resources {
            match resource {
                PreparedProjectResource::Record(metadata) => lazy_records.push(metadata),
                PreparedProjectResource::Ready(metadata, imported) => {
                    ready_resources.push((metadata, imported));
                }
            }
        }
        self.resource_manager.register_lazy_records(lazy_records);
        for (metadata, imported) in ready_resources {
            register_project_resource(&self.resource_manager, metadata, imported);
        }
        *self.project_source_paths_write() = prepared.source_paths;
    }

    fn prepare_incremental_project_resource_sync(
        &self,
        project: &ProjectManager,
        previous_source_records: &[ResourceRecord],
        updated_records: Vec<ResourceRecord>,
    ) -> PreparedIncrementalProjectResourceSync {
        let mut records_by_id = std::collections::HashMap::new();
        for record in updated_records {
            records_by_id.insert(record.id(), record);
        }
        let removed_locators = previous_source_records
            .iter()
            .filter(|previous| {
                project
                    .registry()
                    .get(previous.id())
                    .is_none_or(|current| current.primary_locator != previous.primary_locator)
            })
            .map(|record| record.primary_locator.clone())
            .collect::<Vec<_>>();
        let mut source_path_updates = std::collections::HashMap::new();
        for record in records_by_id.values() {
            let source_uri = AssetUri::new(
                record.primary_locator.scheme(),
                record.primary_locator.path().to_string(),
                None,
            )
            .expect("a parsed resource locator remains valid when its label is removed");
            if let Ok(source_path) = project.source_path_for_uri(&source_uri) {
                source_path_updates.insert(source_uri, source_path);
            }
        }
        let source_path_removals = previous_source_records
            .iter()
            .map(|record| {
                AssetUri::new(
                    record.primary_locator.scheme(),
                    record.primary_locator.path().to_string(),
                    None,
                )
                .expect("a parsed resource locator remains valid when its label is removed")
            })
            .filter(|source_uri| project.source_resource_records(source_uri).is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        PreparedIncrementalProjectResourceSync {
            removed_locators,
            source_path_removals,
            source_path_updates: source_path_updates.into_iter().collect(),
            record_updates: records_by_id.into_values().collect(),
        }
    }

    fn commit_incremental_project_resource_sync(
        &self,
        prepared: PreparedIncrementalProjectResourceSync,
    ) {
        for locator in prepared.removed_locators {
            let _ = self.resource_manager.remove_by_locator(&locator);
        }
        self.resource_manager
            .register_lazy_records(prepared.record_updates);
        let mut source_paths = self.project_source_paths_write();
        for source_uri in prepared.source_path_removals {
            let remove_scheme = source_paths
                .get_mut(&source_uri.scheme())
                .is_some_and(|paths| {
                    paths.remove(source_uri.path());
                    paths.is_empty()
                });
            if remove_scheme {
                source_paths.remove(&source_uri.scheme());
            }
        }
        for (source_uri, source_path) in prepared.source_path_updates {
            source_paths
                .entry(source_uri.scheme())
                .or_default()
                .insert(source_uri.path().to_string(), source_path);
        }
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
        let mut lazy_records = prepared.record_updates;
        let mut ready_resources = Vec::new();
        for resource in prepared.resources {
            match resource {
                PreparedProjectResource::Record(metadata) => lazy_records.push(metadata),
                PreparedProjectResource::Ready(metadata, payload) => {
                    ready_resources.push((metadata, payload));
                }
            }
        }
        self.resource_manager.register_lazy_records(lazy_records);
        for (metadata, payload) in ready_resources {
            register_project_resource(&self.resource_manager, metadata, payload);
        }
        self.project_source_paths_write()
            .entry(prepared.source_uri.scheme())
            .or_default()
            .insert(prepared.source_uri.path().to_string(), prepared.source_path);
    }

    pub(super) fn process_watch_batch_in_generation(&self, batch: AssetWatchBatch) {
        self.record_asset_watch_batch(&batch);
        let AssetWatchBatch {
            changes,
            requires_reconciliation,
            diagnostics: _,
        } = batch;
        if changes.is_empty() && !requires_reconciliation {
            return;
        }

        let use_incremental_sync = !requires_reconciliation
            && ProjectManager::watch_changes_use_incremental_path(&changes);
        let defer_targeted_file_commit = use_incremental_sync;
        let _watch_refresh = self.lock_watch_refresh();
        for attempt in 0..WATCH_GENERATION_RETRY_LIMIT {
            let (
                expected_generation,
                expected_preparation_epoch,
                expected_root,
                watch_error_root,
                previous_locators,
                previous_source_records,
                mut candidate,
            ) = {
                let _generation = self.project_generation_read();
                let project = self.project_read();
                let Some(active_project) = project.as_ref() else {
                    return;
                };
                (
                    active_project.catalog_input_generation().sequence(),
                    self.current_project_preparation_epoch(),
                    active_project.paths().root().to_path_buf(),
                    active_project
                        .primary_project_asset_root()
                        .unwrap_or_else(|_| active_project.paths().root())
                        .to_path_buf(),
                    super::super::resource_sync::project_locators(active_project),
                    changes
                        .iter()
                        .flat_map(|change| {
                            let mut records = active_project.source_resource_records(&change.uri);
                            if let Some(previous_uri) = change.previous_uri.as_ref() {
                                records
                                    .extend(active_project.source_resource_records(previous_uri));
                            }
                            records
                        })
                        .collect::<Vec<_>>(),
                    active_project.clone(),
                )
            };
            let scan_started = std::time::Instant::now();
            let scan_result = if defer_targeted_file_commit {
                candidate
                    .prepare_targeted_watch_changes(&changes)
                    .map(|(records, prepared)| (records, Some(prepared)))
            } else if requires_reconciliation {
                candidate.scan_and_import().map(|records| (records, None))
            } else {
                candidate
                    .scan_and_import_watch_changes(&changes)
                    .map(|records| (records, None))
            };
            self.record_asset_watch_scan(scan_started.elapsed());
            let (updated_records, mut prepared_targeted_generation) = match scan_result {
                Ok(prepared) => prepared,
                Err(error) => {
                    self.record_asset_watch_failure();
                    self.broadcast_watch_error(AssetWatchError::from_message(
                        watch_error_root,
                        error.to_string(),
                    ));
                    return;
                }
            };
            let prepared = match if use_incremental_sync {
                Ok(PreparedWatchProjectResourceSync::Incremental(
                    self.prepare_incremental_project_resource_sync(
                        &candidate,
                        &previous_source_records,
                        updated_records,
                    ),
                ))
            } else {
                self.prepare_project_resource_sync(&candidate)
                    .map(PreparedWatchProjectResourceSync::Reconciliation)
            } {
                Ok(prepared) => prepared,
                Err(error) => {
                    self.record_asset_watch_failure();
                    self.broadcast_watch_error(AssetWatchError::from_message(
                        watch_error_root,
                        error.to_string(),
                    ));
                    return;
                }
            };
            let generation = self.project_generation_write();
            let commit_result = {
                let mut project = self.project_write();
                let Some(active_project) = project.as_mut() else {
                    return;
                };
                if active_project.paths().root() != expected_root {
                    Ok((false, None))
                } else if active_project.catalog_input_generation().sequence()
                    != expected_generation
                    || self.current_project_preparation_epoch() != expected_preparation_epoch
                {
                    Ok((false, None))
                } else {
                    let targeted_commit_result: Result<(), CoreError> =
                        match prepared_targeted_generation.take() {
                            Some(prepared_generation) => {
                                prepared_generation.commit().map_err(asset_error)
                            }
                            None => Ok(()),
                        };
                    match targeted_commit_result {
                        Err(error) => Err(error),
                        Ok(()) => {
                            let incremental_resource_count = match prepared {
                                PreparedWatchProjectResourceSync::Reconciliation(prepared) => {
                                    super::super::resource_sync::clear_removed_project_resources(
                                        &self.resource_manager,
                                        &previous_locators,
                                        &candidate,
                                    );
                                    self.commit_project_resource_sync(prepared);
                                    None
                                }
                                PreparedWatchProjectResourceSync::Incremental(prepared) => {
                                    let count = prepared.record_updates.len();
                                    self.commit_incremental_project_resource_sync(prepared);
                                    Some(count)
                                }
                            };
                            *active_project = candidate;
                            Ok((true, incremental_resource_count))
                        }
                    }
                }
            };
            let (committed, incremental_resource_count) = match commit_result {
                Ok(result) => result,
                Err(error) => {
                    drop(generation);
                    self.record_asset_watch_failure();
                    self.broadcast_watch_error(AssetWatchError::from_message(
                        watch_error_root,
                        error.to_string(),
                    ));
                    return;
                }
            };
            if committed {
                if let Some(count) = incremental_resource_count {
                    self.record_asset_watch_incremental_resource_sync(count);
                }
                self.record_asset_watch_commit();
                drop(_watch_refresh);
                let published_changes = changes
                    .into_iter()
                    .map(|change| AssetChange::new(change.kind, change.uri, change.previous_uri))
                    .collect();
                self.publish_project_generation(generation, published_changes);
                return;
            }
            drop(generation);
            self.record_asset_watch_superseded_generation();
            if attempt + 1 == WATCH_GENERATION_RETRY_LIMIT {
                self.record_asset_watch_failure();
                self.broadcast_watch_error(AssetWatchError::from_message(
                    expected_root,
                    "asset watch refresh was superseded by repeated project generation changes",
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::{mpsc, Arc, TryLockError};
    use std::thread;
    use std::time::Duration;

    use crate::asset::watch::AssetChangeKind;

    use super::*;

    fn watcher_activation(lifecycle: ProjectWatcherLifecycle) -> ProjectWatcherActivation {
        ProjectWatcherActivation {
            state: std::sync::Mutex::new(ProjectWatcherActivationState {
                lifecycle,
                changes: Vec::new(),
                coalescible_change_indices: Default::default(),
                queued_change_bytes: 0,
                requires_reconciliation: false,
                diagnostics: Default::default(),
                errors: Vec::new(),
                worker_scheduled: false,
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
        let activation = std::sync::Arc::new(watcher_activation(ProjectWatcherLifecycle::Pending));
        let manager = ProjectAssetManager::default();

        activation.enqueue_batch(
            &manager,
            AssetWatchBatch {
                changes: vec![watcher_change("res://first.json")],
                ..AssetWatchBatch::default()
            },
        );
        activation.begin_draining();
        activation.enqueue_batch(
            &manager,
            AssetWatchBatch {
                changes: vec![watcher_change("res://second.json")],
                ..AssetWatchBatch::default()
            },
        );

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
            let _guard = manager.watch_refresh_gate.lock().unwrap();
            panic!("poison watch refresh gate");
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
        drop(manager.lock_watch_refresh());
        assert!(manager.lock_watchers().is_empty());
    }

    #[test]
    fn only_the_latest_project_preparation_epoch_can_publish() {
        let manager = ProjectAssetManager::default();

        let older = manager.begin_project_preparation();
        let newer = manager.begin_project_preparation();

        assert!(!manager.is_latest_project_preparation(older));
        assert!(manager.is_latest_project_preparation(newer));
    }

    #[test]
    fn project_generation_publication_holds_the_write_fence_through_broadcast() {
        let manager = Arc::new(ProjectAssetManager::default());
        let subscribers = manager.lock_change_subscribers();
        let (generation_acquired, generation_ready) = mpsc::sync_channel(0);
        let publishing_manager = Arc::clone(&manager);
        let publication = thread::spawn(move || {
            let generation = publishing_manager.project_generation_write();
            generation_acquired.send(()).unwrap();
            publishing_manager.publish_project_generation(
                generation,
                vec![watcher_change("res://generation-fence.json")],
            );
        });

        generation_ready
            .recv_timeout(Duration::from_secs(2))
            .expect("publication worker must acquire the generation fence");
        assert!(matches!(
            manager.project_generation_gate.try_read(),
            Err(TryLockError::WouldBlock)
        ));
        assert!(matches!(
            manager.project_generation_gate.try_write(),
            Err(TryLockError::WouldBlock)
        ));

        drop(subscribers);
        publication.join().unwrap();
    }

    #[test]
    fn generation_publication_callers_share_the_fenced_runtime_owner() {
        let runtime = include_str!("runtime.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("runtime production source must precede its test module");
        let close = include_str!("close_project.rs");
        let contract = include_str!("../service_contracts/asset_manager_contract.rs");

        assert_eq!(
            close.matches("self.publish_project_generation(").count(),
            1,
            "close must publish through the generation-fenced owner"
        );
        assert_eq!(
            contract.matches("self.publish_project_generation(").count(),
            2,
            "targeted import and full reimport must share the fenced owner"
        );
        assert_eq!(
            runtime.matches("self.publish_project_generation(").count(),
            1,
            "watcher commit must publish through the same fenced owner"
        );
        assert!(!contract.contains("drop(_generation);"));
        assert!(!close.contains("drop(generation);"));
    }
}
