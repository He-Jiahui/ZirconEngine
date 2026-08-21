use std::collections::hash_map::DefaultHasher;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;
use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use crate::asset::project::ProjectGenerationPhase;
use crate::asset::{AssetImporterRegistry, AssetUri, ProjectManager};
use crate::core::CoreError;
use crate::core::framework::channel::{ChannelSender, ChannelWakeCallback};

use super::super::errors::asset_error;
use super::ProjectAssetManager;
use super::project_asset_manager::{
    ProjectAssetChangeSubscriber, ProjectAssetGenerationWakeSubscriber, ProjectSourcePathIndex,
    ProjectWatcherActivation, ProjectWatcherActivationState, ProjectWatcherLifecycle,
};
use super::resource_publication::PreparedWatchProjectResourceSync;
use crate::asset::watch::{AssetChange, AssetWatchBatch, AssetWatchError, AssetWatcher};
use crate::core::resource::{ResourceMutationBatch, ResourceRecord, ResourceScheme};

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

    pub(super) fn project_source_paths_write(
        &self,
    ) -> RwLockWriteGuard<'_, ProjectSourcePathIndex> {
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

    pub(super) fn build_project_source_paths(
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
    ) -> MutexGuard<'_, Vec<ProjectAssetChangeSubscriber>> {
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

    fn lock_generation_wake_subscribers(
        &self,
    ) -> MutexGuard<'_, Vec<ProjectAssetGenerationWakeSubscriber>> {
        self.generation_wake_subscribers
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
        subscribers.retain(|subscriber| {
            let delivered = changes.iter().all(|change| subscriber.send(change.clone()));
            if delivered {
                subscriber.wake();
            }
            delivered
        });
        drop(subscribers);
    }

    fn publish_generation_wake(&self) {
        let wakes = {
            let mut subscribers = self.lock_generation_wake_subscribers();
            let mut wakes = Vec::new();
            subscribers.retain(|subscriber| match subscriber.try_enqueue() {
                Some(true) => {
                    wakes.push(subscriber.wake_callback());
                    true
                }
                Some(false) => true,
                None => false,
            });
            wakes
        };
        for wake in wakes {
            wake();
        }
    }

    pub(crate) fn subscribe_project_generation_wake(
        &self,
        wake: ChannelWakeCallback,
    ) -> crate::core::framework::channel::ChannelReceiver<()> {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        self.lock_generation_wake_subscribers()
            .push(ProjectAssetGenerationWakeSubscriber::new(sender, wake));
        receiver
    }

    pub(in crate::asset::pipeline::manager) fn subscribe_asset_changes_internal(
        &self,
        wake: Option<ChannelWakeCallback>,
    ) -> crate::core::framework::channel::ChannelReceiver<AssetChange> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.lock_change_subscribers()
            .push(ProjectAssetChangeSubscriber::new(sender, wake));
        receiver
    }

    pub(in crate::asset::pipeline::manager) fn publish_project_generation(
        &self,
        generation: RwLockWriteGuard<'_, ()>,
        changes: Vec<AssetChange>,
    ) {
        let _phase = ProjectGenerationPhase::GenerationPublish.enter();
        self.broadcast(changes);
        self.publish_generation_wake();
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
                errors: Default::default(),
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
            let scan_result =
                candidate.prepare_watch_file_generation(&changes, use_incremental_sync);
            self.record_asset_watch_scan(scan_started.elapsed());
            let (updated_records, prepared_file_generation) = match scan_result {
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
            let mut prepared_file_generation = Some(prepared_file_generation);
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
                let Some(active_project) = project.as_ref() else {
                    return;
                };
                if active_project.paths().root() != expected_root {
                    Ok((false, None, None))
                } else if active_project.catalog_input_generation().sequence()
                    != expected_generation
                    || self.current_project_preparation_epoch() != expected_preparation_epoch
                {
                    Ok((false, None, None))
                } else {
                    let resource_sync = match prepared {
                        PreparedWatchProjectResourceSync::Reconciliation(prepared) => {
                            let batch =
                                super::super::resource_sync::clear_removed_project_resources(
                                    ResourceMutationBatch::new(),
                                    &previous_locators,
                                    &candidate,
                                );
                            self.commit_project_resource_sync(
                                prepared,
                                batch,
                                || {
                                    prepared_file_generation
                                        .take()
                                        .expect("one prepared file generation commits once")
                                        .commit()
                                        .map_err(asset_error)
                                },
                                || {
                                    *project = Some(candidate);
                                    drop(project);
                                },
                            )
                            .map(|outcome| (None, outcome))
                        }
                        PreparedWatchProjectResourceSync::Incremental(prepared) => {
                            let count = prepared.record_updates.len();
                            self.commit_incremental_project_resource_sync(
                                prepared,
                                || {
                                    prepared_file_generation
                                        .take()
                                        .expect("one prepared file generation commits once")
                                        .commit()
                                        .map_err(asset_error)
                                },
                                || {
                                    *project = Some(candidate);
                                    drop(project);
                                },
                            )
                            .map(|outcome| (Some(count), outcome))
                        }
                    };
                    resource_sync.map(|(incremental_resource_count, outcome)| {
                        (true, incremental_resource_count, Some(outcome))
                    })
                }
            };
            let (committed, incremental_resource_count, commit_outcome) = match commit_result {
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
                if let Some(outcome) = commit_outcome {
                    if let Err(error) = outcome.ensure_durable() {
                        self.record_asset_watch_failure();
                        self.broadcast_watch_error(AssetWatchError::from_message(
                            watch_error_root,
                            error.to_string(),
                        ));
                    }
                }
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
mod tests;
