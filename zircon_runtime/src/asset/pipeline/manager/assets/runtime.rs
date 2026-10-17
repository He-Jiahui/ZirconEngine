use std::collections::hash_map::DefaultHasher;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard, TryLockError};
use std::time::Instant;

use crate::asset::project::{
    ImportSourceWatchEcho, ProjectActivationIdentity, ProjectGenerationPhase,
};
use crate::asset::{AssetImporterRegistry, AssetUri, ProjectManager};
use crate::core::framework::channel::ChannelWakeCallback;
use crate::core::CoreError;

use super::super::errors::asset_error;
use super::management_generation::ProjectAssetManagementGeneration;
use super::project_asset_manager::{
    ProjectAssetGenerationWakeSubscriber, ProjectSourcePathIndex, ProjectWatcherActivation,
    ProjectWatcherActivationState, ProjectWatcherLifecycle,
};
use super::resource_publication::PreparedWatchProjectResourceSync;
use super::source_write_watch_echo::TransactionWatchEchoes;
use super::ProjectAssetManager;
use crate::asset::watch::{AssetChange, AssetWatchBatch, AssetWatchError, AssetWatcher};
use crate::core::resource::{
    ResourceManagerAssemblyExt, ResourceMutationBatch, ResourceScheme,
};

pub(in crate::asset::pipeline::manager) struct PreparedProjectWatchers {
    watchers: Vec<AssetWatcher>,
    activation: std::sync::Arc<ProjectWatcherActivation>,
}

const WATCH_GENERATION_RETRY_LIMIT: usize = 3;

/// Owned preparation only: no authority, serial, project or file-commit guard crosses a retry.
pub(super) struct DeferredWatchPublication {
    pub(super) batch: AssetWatchBatch,
    previous_project: ProjectManager,
    candidate: Option<ProjectManager>,
    files: Option<crate::asset::project::PreparedWatchFileGeneration>,
    sources: Vec<crate::asset::project::WatchSourceCandidate>,
    geometry_demand: Arc<crate::core::resource::ResourceActiveGeneration>,
    sync: Option<PreparedWatchProjectResourceSync>,
    expected_generation: u64,
    expected_preparation_epoch: u64,
    activation: ProjectActivationIdentity,
    project_root: PathBuf,
    watch_error_root: PathBuf,
}

pub(super) enum WatchRetryWork {
    Prepared(DeferredWatchPublication),
    Reprepare(AssetWatchBatch),
}

impl WatchRetryWork {
    pub(super) fn take_batch(&mut self) -> AssetWatchBatch {
        match self {
            Self::Prepared(work) => std::mem::take(&mut work.batch),
            Self::Reprepare(batch) => std::mem::take(batch),
        }
    }
}

/// An immutable view of the active project and the identity required to commit work derived from
/// it. Expensive preparation may use the cloned project without retaining the generation gate.
#[derive(Clone, Debug)]
pub struct ProjectAssetGenerationSnapshot {
    project: ProjectManager,
    token: ProjectAssetGenerationToken,
}

impl ProjectAssetGenerationSnapshot {
    pub fn project(&self) -> &ProjectManager {
        &self.project
    }

    pub fn into_parts(self) -> (ProjectManager, ProjectAssetGenerationToken) {
        (self.project, self.token)
    }
}

/// Runtime-issued identity for work prepared from one active project generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectAssetGenerationToken {
    activation_identity: ProjectActivationIdentity,
    project_root: PathBuf,
    catalog_sequence: u64,
}

impl ProjectAssetGenerationToken {
    pub fn activation_identity(&self) -> &ProjectActivationIdentity {
        &self.activation_identity
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProjectGenerationCommitOutcome<T> {
    Committed(T),
    Superseded { newer_same_project_generation: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectGenerationMatch {
    Current,
    Superseded { newer_same_project_generation: bool },
}

fn project_generation_match(
    project: Option<&ProjectManager>,
    expected: &ProjectAssetGenerationToken,
) -> ProjectGenerationMatch {
    let same_project =
        project.is_some_and(|project| project.paths().root() == expected.project_root);
    let current_catalog_sequence =
        project.map(|project| project.catalog_input_generation().sequence());
    let same_activation = project
        .is_some_and(|project| project.activation_identity() == &expected.activation_identity);
    if same_activation
        && same_project
        && current_catalog_sequence == Some(expected.catalog_sequence)
    {
        ProjectGenerationMatch::Current
    } else {
        ProjectGenerationMatch::Superseded {
            newer_same_project_generation: same_project
                && current_catalog_sequence != Some(expected.catalog_sequence),
        }
    }
}

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

    fn retire(&self) { self.retire_retry(None); }

    fn try_retire(&self) -> bool { self.try_retire_until(Instant::now()) }

    fn try_retire_until(&self, deadline: Instant) -> bool {
        self.retire_retry(Some(deadline))
    }

    pub(super) fn retire_locked(state: &mut ProjectWatcherActivationState) {
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
    /// Checks the active packet identity under the existing generation gate.
    /// This is an observation, not a permit to import within a read-lock callback.
    pub fn is_project_activation_current(&self, expected: &ProjectActivationIdentity) -> bool {
        let _generation = self.project_generation_read();
        self.project_read()
            .as_ref()
            .is_some_and(|project| project.activation_identity() == expected)
    }

    /// Captures the active project and its commit identity under one generation read fence.
    pub fn current_project_generation_snapshot(&self) -> Option<ProjectAssetGenerationSnapshot> {
        let _generation = self.project_generation_read();
        let project = self.project_read();
        let project = project.as_ref()?;
        let token = ProjectAssetGenerationToken {
            activation_identity: project.activation_identity().clone(),
            project_root: project.paths().root().to_path_buf(),
            catalog_sequence: project.catalog_input_generation().sequence(),
        };
        Some(ProjectAssetGenerationSnapshot {
            project: project.clone(),
            token,
        })
    }

    /// Classifies prepared work without holding the generation fence across caller preparation.
    /// A terminal mutation must still use `commit_if_project_generation` after this precheck.
    pub fn check_project_generation(
        &self,
        expected: &ProjectAssetGenerationToken,
    ) -> ProjectGenerationMatch {
        let _generation = self.project_generation_read();
        let project = self.project_read();
        project_generation_match(project.as_ref(), expected)
    }

    /// Commits prepared work only while its source generation is still active.
    ///
    /// The callback must stay short and must not perform file or asset preparation. The retained
    /// generation read fence prevents a newer project generation from publishing between the
    /// identity check and the callback's terminal state transition.
    pub fn commit_if_project_generation<T>(
        &self,
        expected: &ProjectAssetGenerationToken,
        commit: impl FnOnce() -> T,
    ) -> ProjectGenerationCommitOutcome<T> {
        let generation = self.project_generation_read();
        let project = self.project_read();
        let generation_match = project_generation_match(project.as_ref(), expected);
        drop(project);
        if let ProjectGenerationMatch::Superseded {
            newer_same_project_generation,
        } = generation_match
        {
            drop(generation);
            return ProjectGenerationCommitOutcome::Superseded {
                newer_same_project_generation,
            };
        }
        let committed = commit();
        drop(generation);
        ProjectGenerationCommitOutcome::Committed(committed)
    }

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

    pub(crate) fn current_asset_management_generation(
        &self,
    ) -> Arc<ProjectAssetManagementGeneration> {
        let _generation = self.project_generation_read();
        self.asset_management_generation_snapshot()
    }

    pub(super) fn asset_management_generation_snapshot(
        &self,
    ) -> Arc<ProjectAssetManagementGeneration> {
        self.asset_management_generation
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub(super) fn install_asset_management_generation(
        &self,
        generation: ProjectAssetManagementGeneration,
    ) {
        *self
            .asset_management_generation
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Arc::new(generation);
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

    fn broadcast(&self, changes: Vec<AssetChange>) -> Vec<ChannelWakeCallback> {
        self.change_events
            .publish(changes, crate::asset::watch::change_bytes)
    }

    fn collect_generation_wakes(&self) -> Vec<ChannelWakeCallback> {
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
        wakes
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
    ) -> crate::asset::watch::AssetChangeSubscription {
        crate::asset::watch::AssetChangeSubscription::new(
            self.change_events.subscribe(wake),
            &self.project_generation_gate,
            &self.asset_management_generation,
            &self.project,
        )
    }

    pub(in crate::asset::pipeline::manager) fn publish_project_generation(
        &self,
        generation: RwLockWriteGuard<'_, ()>,
        changes: Vec<AssetChange>,
    ) {
        let _phase = ProjectGenerationPhase::GenerationPublish.enter();
        self.refresh_asset_management_generation();
        let mut wakes = self.broadcast(changes);
        wakes.extend(self.collect_generation_wakes());
        drop(generation);
        for wake in wakes {
            wake();
        }
    }

    pub(in crate::asset::pipeline::manager) fn broadcast_watch_error(
        &self,
        error: AssetWatchError,
    ) {
        let wakes = self
            .watch_errors
            .publish(std::iter::once(error), crate::asset::watch::error_bytes);
        for wake in wakes {
            wake();
        }
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
                ingress_epoch: Arc::new(()),
                retry_timer: None,
                retry_work: None,
                retry_ready: false,
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
        _publication: &super::watcher_lifecycle::ProjectWatcherPublication<'_>,
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

    /// Stops and joins every watcher without discarding unfinished join authority.
    ///
    /// Closing admission is permanent. An in-flight project operation, including a synchronous
    /// publication callback, returns pending so its owner can finish before a later retry.
    pub fn shutdown_project_watchers_until(&self, deadline: Instant) -> bool {
        if !self.watcher_admission.try_close() {
            return false;
        }
        let mut active = match self.watcher_activation.try_lock() {
            Ok(active) => active,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return false,
        };
        let mut watchers = match self.watchers.try_lock() {
            Ok(watchers) => watchers,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return false,
        };
        if active
            .as_ref()
            .is_some_and(|activation| !activation.try_retire())
        {
            return false;
        }
        active.take();

        // Keep join authority visible until completion; another reaper must not see an empty set.
        while let Some(watcher) = watchers.last_mut() {
            if !watcher.shutdown_until(deadline) {
                return false;
            }
            watchers.pop();
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn install_project_watcher_for_test(&self, watcher: AssetWatcher) {
        self.lock_watchers().push(watcher);
    }

    pub(in crate::asset::pipeline::manager) fn clear_project_source_paths(&self) {
        self.project_source_paths_write().clear();
    }

    pub(in crate::asset::pipeline::manager) fn register_transaction_watch_echoes(
        &self,
        echoes: impl IntoIterator<Item = ImportSourceWatchEcho>,
    ) {
        self.lock_transaction_watch_echoes().register(echoes);
    }

    pub(in crate::asset::pipeline::manager) fn clear_transaction_watch_echoes(&self) {
        self.lock_transaction_watch_echoes().clear();
    }

    pub(super) fn process_watch_batch_in_generation(&self, batch: AssetWatchBatch) {
        let activation = self.watcher_activation.lock().unwrap_or_else(|p| p.into_inner()).clone();
        let ingress = activation.as_ref().map(|activation| Arc::clone(&activation.lock_state().ingress_epoch));
        let epoch = activation.as_deref().zip(ingress.as_ref());
        if let Some(work) = self.prepare_watch_retry_work(batch, epoch) {
            if let (Some(activation), Some(ingress)) = (activation, ingress) {
                activation.defer_watch_work_at(self, work, ingress);
            }
        }
    }

    pub(super) fn process_watch_retry_work(
        &self, work: WatchRetryWork,
        activation: &ProjectWatcherActivation, ingress: &Arc<()>,
    ) -> Option<WatchRetryWork> {
        match work {
            WatchRetryWork::Prepared(work) => self.commit_watch_preparation(work, Some((activation, ingress))),
            WatchRetryWork::Reprepare(batch) => self.prepare_watch_retry_work(batch, Some((activation, ingress))),
        }
    }

    fn prepare_watch_retry_work(
        &self, batch: AssetWatchBatch,
        epoch: Option<(&ProjectWatcherActivation, &Arc<()>)>,
    ) -> Option<WatchRetryWork> {
        let work = self.prepare_watch_publication(batch)?;
        self.commit_watch_preparation(work, epoch)
    }

    fn prepare_watch_publication(&self, batch: AssetWatchBatch) -> Option<DeferredWatchPublication> {
        self.record_asset_watch_batch(&batch);
        let changes = self.lock_transaction_watch_echoes().filter(batch.changes);
        if changes.is_empty() && !batch.requires_reconciliation { return None; }
        let batch = AssetWatchBatch {
            changes, requires_reconciliation: batch.requires_reconciliation,
            diagnostics: Default::default(),
        };
        let previous_project = {
            let _generation = self.project_generation_read();
            self.project_read().as_ref()?.clone()
        };
        let project_root = previous_project.paths().root().to_path_buf();
        let watch_error_root = previous_project.primary_project_asset_root()
            .unwrap_or_else(|_| previous_project.paths().root()).to_path_buf();
        let expected_generation = previous_project.catalog_input_generation().sequence();
        let activation = previous_project.activation_identity().clone();
        let expected_preparation_epoch = self.current_project_preparation_epoch();
        let mut candidate = previous_project.clone();
        let geometry_demand = Arc::clone(self.resource_manager.projection_snapshot().active());
        let use_incremental = !batch.requires_reconciliation
            && ProjectManager::watch_changes_use_incremental_path(&batch.changes);
        let preparation = (|| {
            let _project_generation = crate::asset::project::lock_project_generation(&project_root)?;
            let _refresh = self.lock_watch_refresh();
            let started = Instant::now();
            let result = candidate.prepare_watch_file_generation_with_geometry_demand(
                &batch.changes, use_incremental, Some(&geometry_demand),
            );
            self.record_asset_watch_scan(started.elapsed());
            result
        })();
        let (updated_records, mut files) = match preparation {
            Ok(result) => result,
            Err(error) => {
                self.record_asset_watch_failure();
                self.broadcast_watch_error(AssetWatchError::from_message(watch_error_root, error.to_string()));
                return None;
            }
        };
        let sources = files.take_sources(&geometry_demand);
        // The primary source manifest owns auxiliary-buffer events as well as direct events.
        let previous_source_records = sources.iter().flat_map(|source| source.previous_members.iter())
            .filter_map(|id| previous_project.registry().get(*id).cloned()).collect::<Vec<_>>();
        let sync = if use_incremental {
            Ok(PreparedWatchProjectResourceSync::Incremental(
                self.prepare_incremental_project_resource_sync(&candidate, &previous_source_records, updated_records),
            ))
        } else {
            self.prepare_project_resource_sync(&candidate).map(PreparedWatchProjectResourceSync::Reconciliation)
        };
        let sync = match sync {
            Ok(sync) => sync,
            Err(error) => {
                self.record_asset_watch_failure();
                self.broadcast_watch_error(AssetWatchError::from_message(watch_error_root, error.to_string()));
                return None;
            }
        };
        Some(DeferredWatchPublication {
            batch, previous_project, candidate: Some(candidate), files: Some(files), sources, geometry_demand,
            sync: Some(sync), expected_generation, expected_preparation_epoch, activation,
            project_root, watch_error_root,
        })
    }

    fn commit_watch_preparation(
        &self, mut work: DeferredWatchPublication,
        epoch: Option<(&ProjectWatcherActivation, &Arc<()>)>,
    ) -> Option<WatchRetryWork> {
        let project_generation = match crate::asset::project::lock_project_generation(&work.project_root) {
            Ok(guard) => guard,
            Err(error) => {
                self.record_asset_watch_failure();
                self.broadcast_watch_error(AssetWatchError::from_message(work.watch_error_root, error.to_string()));
                return None;
            }
        };
        let refresh = self.lock_watch_refresh();
        for _ in 0..WATCH_GENERATION_RETRY_LIMIT {
            let projection = self.resource_manager.projection_snapshot();
            if work.sources.iter().any(|source| source.requires_geometry_reprepare(&work.geometry_demand, projection.active())) {
                // A cold member became resident after preparation. Its output was intentionally
                // released, so recapture must re-import rather than retry this incomplete output.
                self.record_asset_watch_superseded_generation();
                drop(refresh);
                drop(project_generation);
                return Some(WatchRetryWork::Reprepare(work.batch));
            }
            let candidate = work.candidate.as_ref().expect("uncommitted candidate");
            let sync = work.sync.as_ref().expect("uncommitted source paths");
            let mut batch = ResourceMutationBatch::new();
            if matches!(sync, PreparedWatchProjectResourceSync::Reconciliation(_)) {
                batch = super::super::resource_sync::reconcile_project_resources(
                    batch,
                    &super::super::resource_sync::project_resource_identities(&work.previous_project),
                    candidate,
                );
            }
            batch = self.build_watch_resource_batch(sync, &work.sources, projection.active(), batch);
            let generation = self.project_generation_write();
            let mut project = self.project_write();
            let Some(current) = project.as_ref() else { return None; };
            if current.activation_identity() != &work.activation || current.paths().root() != work.project_root {
                return None;
            }
            if current.catalog_input_generation().sequence() != work.expected_generation
                || self.current_project_preparation_epoch() != work.expected_preparation_epoch
            {
                drop(project);
                drop(generation);
                drop(refresh);
                drop(project_generation);
                self.record_asset_watch_superseded_generation();
                return Some(WatchRetryWork::Reprepare(work.batch));
            }
            if let Some((activation, ingress)) = epoch {
                let state = activation.lock_state();
                if state.lifecycle == ProjectWatcherLifecycle::Retired { return None; }
                if !Arc::ptr_eq(&state.ingress_epoch, ingress) {
                    // A new event owns the latest FS preparation; the previous owned candidate
                    // must not be replayed after it. Carry only event responsibility forward.
                    drop(state);
                    drop(project);
                    drop(generation);
                    return Some(WatchRetryWork::Reprepare(work.batch));
                }
            }
            let resource = match self.resource_manager.prepare_commit(batch) {
                Ok(resource) => resource,
                Err(crate::core::resource::ResourceRegistryError::ActiveVersionConflict { .. }) => {
                    drop(project);
                    drop(generation);
                    self.record_asset_watch_superseded_generation();
                    continue;
                }
                Err(error) => {
                    drop(project);
                    drop(generation);
                    self.record_asset_watch_failure();
                    self.broadcast_watch_error(AssetWatchError::from_message(work.watch_error_root, error.to_string()));
                    return None;
                }
            };
            let outcome = match work.files.take().expect("one file generation commits once").commit() {
                Ok(outcome) => outcome,
                Err(error) => {
                    // Dropping the prepared resource reservation publishes nothing.
                    drop(resource);
                    drop(project);
                    drop(generation);
                    self.record_asset_watch_failure();
                    self.broadcast_watch_error(AssetWatchError::from_message(work.watch_error_root, error.to_string()));
                    return None;
                }
            };
            let incremental_count = match work.sync.as_ref().expect("uncommitted paths") {
                PreparedWatchProjectResourceSync::Incremental(sync) => Some(sync.record_updates.len()),
                _ => None,
            };
            self.install_watch_resource_paths(work.sync.take().expect("one source-path installation"));
            *project = work.candidate.take();
            drop(project);
            resource.commit();
            if let Some(count) = incremental_count { self.record_asset_watch_incremental_resource_sync(count); }
            self.record_asset_watch_commit();
            drop(refresh);
            drop(project_generation);
            let published_changes = work.batch.changes.into_iter()
                .map(|change| AssetChange::new(change.kind, change.uri, change.previous_uri)).collect();
            self.publish_project_generation(generation, published_changes);
            if let Err(error) = outcome.ensure_durable() {
                self.record_asset_watch_failure();
                self.broadcast_watch_error(AssetWatchError::from_message(work.watch_error_root, error.to_string()));
            }
            return None;
        }
        drop(refresh);
        drop(project_generation);
        self.broadcast_watch_error(AssetWatchError::from_message(
            work.watch_error_root.clone(),
            "asset watch active publication deferred; the same activation will retry automatically",
        ));
        Some(WatchRetryWork::Prepared(work))
    }

    fn lock_transaction_watch_echoes(&self) -> MutexGuard<'_, TransactionWatchEchoes> {
        self.transaction_watch_echoes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
#[path = "runtime/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../manager/assets/runtime/tests/watcher_shutdown.rs"]
mod watcher_shutdown_tests;
