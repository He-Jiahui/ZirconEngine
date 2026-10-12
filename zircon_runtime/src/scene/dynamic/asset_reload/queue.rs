#[path = "queue/event_processing.rs"]
mod event_processing;
#[path = "queue/order.rs"]
mod order;
#[path = "queue/ready_collection.rs"]
mod ready_collection;
#[path = "queue/reconciliation.rs"]
mod reconciliation;
#[path = "queue/target_staging.rs"]
mod target_staging;

use std::{collections::HashMap, time::Instant};

use crate::{
    asset::{
        facade::AssetEventPoll, project::ProjectManager, AssetEvent, AssetEventReceiver, AssetId,
        AssetUri, ProjectAssetManager, SceneAsset,
    },
    core::{
        framework::channel::{ChannelReceiver, ChannelWakeCallback},
        manager::{resolve_manager_service, ManagerServiceHandle},
        resource::ResourceManager,
        CoreError, CoreHandle, CoreResult, CoreWeak, JobScheduler, ServiceCallGuard,
        TaskGraphScope,
    },
    scene::LevelSystem,
};

#[cfg(test)]
use crate::scene::World;

use super::{
    diagnostics::DynamicSceneAssetReloadDiagnostics, limits::DynamicSceneAssetReloadLimits,
    reports::DynamicSceneAssetReloadFrameApplyReport, result::DynamicSceneAssetReloadResult,
    stage_task::DynamicSceneAssetReloadStageTask, task::DynamicSceneAssetReloadTask,
};
use order::AssetIdOrder;
use reconciliation::DynamicSceneAssetReloadReconciliation;

const LATEST_REVISION_METADATA_BYTES: usize = std::mem::size_of::<(AssetId, LatestRevisionState)>();
const ORDER_ENTRY_METADATA_BYTES: usize = std::mem::size_of::<AssetId>() * 3;

#[derive(Clone, Copy, Debug)]
struct LatestRevisionState {
    revision: u64,
    authority_rank: u8,
    observed_at: Instant,
}

#[derive(Debug)]
struct DeferredReload {
    event: AssetEvent<SceneAsset>,
    uri: AssetUri,
    label: String,
    metadata_bytes: usize,
    queued_at: Instant,
}

enum DynamicSceneReloadProjectSource {
    AssetManager {
        core: CoreWeak,
        handle: ManagerServiceHandle<ProjectAssetManager>,
    },
    #[cfg(test)]
    Static,
}

impl DynamicSceneReloadProjectSource {
    fn resolve(&self) -> CoreResult<Option<ServiceCallGuard<ProjectAssetManager>>> {
        match self {
            Self::AssetManager { core, handle } => {
                let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
                resolve_manager_service(&core, handle.clone()).map(Some)
            }
            #[cfg(test)]
            Self::Static => Ok(None),
        }
    }
}

pub struct DynamicSceneAssetReloadQueue {
    project: ProjectManager,
    project_source: DynamicSceneReloadProjectSource,
    source_error: Option<CoreError>,
    /// Terminal for this queue instance; old target-bound work is retained until disposal.
    lifecycle_interruption: Option<crate::scene::DynamicSceneError>,
    source_failure_report: Option<super::reports::DynamicSceneAssetReloadDrainReport>,
    catalog_generation_sequence: u64,
    resource_manager: ResourceManager,
    events: AssetEventReceiver<SceneAsset>,
    runtime_frame_wake_token: Option<ChannelReceiver<()>>,
    task_graph_scope: Option<TaskGraphScope>,
    carried_event: Option<AssetEventPoll<SceneAsset>>,
    reconciliation: Option<DynamicSceneAssetReloadReconciliation>,
    pending: HashMap<AssetId, DynamicSceneAssetReloadTask>,
    /// Physical workers retain their permit here until the worker actually exits.
    pending_order: AssetIdOrder,
    /// At most one logical successor is retained for each superseded physical worker.
    deferred: HashMap<AssetId, DeferredReload>,
    deferred_order: AssetIdOrder,
    ready: HashMap<AssetId, DynamicSceneAssetReloadResult>,
    ready_order: AssetIdOrder,
    target_staging: HashMap<AssetId, DynamicSceneAssetReloadStageTask>,
    target_staging_order: AssetIdOrder,
    latest_revisions: HashMap<AssetId, LatestRevisionState>,
    latest_order: AssetIdOrder,
    pending_metadata_bytes: usize,
    ready_result_bytes: usize,
    limits: DynamicSceneAssetReloadLimits,
    diagnostics: DynamicSceneAssetReloadDiagnostics,
}

impl DynamicSceneAssetReloadQueue {
    #[cfg(test)]
    pub(crate) fn new(
        project: ProjectManager,
        events: AssetEventReceiver<SceneAsset>,
        resource_manager: ResourceManager,
    ) -> Self {
        Self::with_limits(
            project,
            events,
            resource_manager,
            DynamicSceneAssetReloadLimits::default(),
        )
    }

    #[cfg(test)]
    pub(crate) fn with_limits(
        project: ProjectManager,
        events: AssetEventReceiver<SceneAsset>,
        resource_manager: ResourceManager,
        limits: DynamicSceneAssetReloadLimits,
    ) -> Self {
        Self::construct(
            project,
            DynamicSceneReloadProjectSource::Static,
            events,
            resource_manager,
            limits,
        )
    }

    fn construct(
        project: ProjectManager,
        project_source: DynamicSceneReloadProjectSource,
        events: AssetEventReceiver<SceneAsset>,
        resource_manager: ResourceManager,
        limits: DynamicSceneAssetReloadLimits,
    ) -> Self {
        let catalog_generation_sequence = project.catalog_input_generation().sequence();
        Self {
            project,
            project_source,
            source_error: None,
            lifecycle_interruption: None,
            source_failure_report: None,
            catalog_generation_sequence,
            resource_manager,
            events,
            runtime_frame_wake_token: None,
            task_graph_scope: None,
            carried_event: None,
            reconciliation: None,
            pending: HashMap::new(),
            pending_order: AssetIdOrder::default(),
            deferred: HashMap::new(),
            deferred_order: AssetIdOrder::default(),
            ready: HashMap::new(),
            ready_order: AssetIdOrder::default(),
            target_staging: HashMap::new(),
            target_staging_order: AssetIdOrder::default(),
            latest_revisions: HashMap::new(),
            latest_order: AssetIdOrder::default(),
            pending_metadata_bytes: 0,
            ready_result_bytes: 0,
            limits: limits.normalized(),
            diagnostics: DynamicSceneAssetReloadDiagnostics::default(),
        }
    }

    pub fn from_project_asset_manager(
        project: ProjectManager,
        core: &CoreHandle,
        handle: ManagerServiceHandle<ProjectAssetManager>,
    ) -> CoreResult<Self> {
        Self::from_project_asset_manager_with_limits(
            project,
            core,
            handle,
            DynamicSceneAssetReloadLimits::default(),
        )
    }

    pub fn from_project_asset_manager_with_limits(
        project: ProjectManager,
        core: &CoreHandle,
        handle: ManagerServiceHandle<ProjectAssetManager>,
        limits: DynamicSceneAssetReloadLimits,
    ) -> CoreResult<Self> {
        let asset_manager = resolve_manager_service(core, handle.clone())?;
        Ok(Self::construct(
            project,
            DynamicSceneReloadProjectSource::AssetManager {
                core: core.downgrade(),
                handle,
            },
            asset_manager.subscribe_asset_events::<SceneAsset>(),
            asset_manager.resource_manager(),
            limits,
        ))
    }

    pub(crate) fn with_task_graph_scope(mut self, task_graph_scope: TaskGraphScope) -> Self {
        self.task_graph_scope = Some(task_graph_scope);
        self
    }

    pub(crate) fn install_runtime_frame_wake(
        &mut self,
        wake: ChannelWakeCallback,
    ) -> CoreResult<()> {
        if self.lifecycle_interruption.is_some() {
            // No new wake is needed. Rebinding a source must not retire retained work.
            return Ok(());
        }
        if let Some(error) = &self.source_error {
            return Err(error.clone());
        }
        let subscription = self
            .project_source
            .resolve()
            .map(|manager| manager.map(|manager| manager.subscribe_project_generation_wake(wake)));
        match subscription {
            Ok(subscription) => {
                self.runtime_frame_wake_token = subscription;
                Ok(())
            }
            Err(error) => Err(self.fail_project_source(error)),
        }
    }

    #[cfg(test)]
    pub(crate) fn assert_no_admitted_reload_storage(&self) {
        assert!(self.pending.is_empty());
        assert!(self.pending_order.is_empty());
        assert!(self.deferred.is_empty());
        assert!(self.deferred_order.is_empty());
        assert!(self.ready.is_empty());
        assert!(self.ready_order.is_empty());
        assert!(self.target_staging.is_empty());
        assert!(self.target_staging_order.is_empty());
        assert_eq!(
            self.pending_metadata_bytes,
            self.latest_revisions.len() * LATEST_REVISION_METADATA_BYTES
                + self.latest_order.len() * ORDER_ENTRY_METADATA_BYTES
        );
    }

    /// A live World or Schedule replacement freezes this queue until it is disposed and recreated.
    /// Preflight target changes and ordinary preparation failures do not enter this state.
    pub fn lifecycle_interruption(&self) -> Option<&crate::scene::DynamicSceneError> {
        self.lifecycle_interruption.as_ref()
    }

    fn interrupted_drain_report(&self) -> super::reports::DynamicSceneAssetReloadDrainReport {
        super::reports::DynamicSceneAssetReloadDrainReport {
            lifecycle_interruption: self.lifecycle_interruption.clone(),
            pending_count: self.pending_count(),
            pending_metadata_bytes: self.pending_metadata_bytes,
            ..Default::default()
        }
    }

    fn interrupted_apply_report(&self) -> super::reports::DynamicSceneAssetReloadApplyReport {
        super::reports::DynamicSceneAssetReloadApplyReport {
            lifecycle_interruption: self.lifecycle_interruption.clone(),
            pending_count: self.pending_count(),
            deferred_count: self.target_staging.len(),
            ..Default::default()
        }
    }

    fn interrupted_frame_report(&self) -> DynamicSceneAssetReloadFrameApplyReport {
        DynamicSceneAssetReloadFrameApplyReport {
            drain: self.interrupted_drain_report(),
            apply: self.interrupted_apply_report(),
        }
    }

    #[cfg(test)]
    pub(crate) fn target_stages_ready(&self) -> bool {
        self.target_staging
            .values()
            .all(DynamicSceneAssetReloadStageTask::is_ready)
    }

    pub fn limits(&self) -> DynamicSceneAssetReloadLimits {
        self.limits
    }

    pub fn diagnostics(&self) -> DynamicSceneAssetReloadDiagnostics {
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.target_staging_reserved_bytes = self.target_staging_reserved_bytes();
        diagnostics.resident_result_bytes = diagnostics
            .ready_result_bytes
            .saturating_add(diagnostics.target_staging_reserved_bytes);
        diagnostics.oldest_active_age = self
            .pending
            .values()
            .map(DynamicSceneAssetReloadTask::age)
            .chain(
                self.deferred
                    .values()
                    .map(|reload| reload.queued_at.elapsed()),
            )
            .chain(self.ready.values().map(DynamicSceneAssetReloadResult::age))
            .chain(
                self.target_staging
                    .values()
                    .map(DynamicSceneAssetReloadStageTask::age),
            )
            .max()
            .unwrap_or_default();
        diagnostics
    }

    #[cfg(test)]
    pub(crate) fn tick_into(
        &mut self,
        scheduler: &JobScheduler,
        world: &mut World,
    ) -> DynamicSceneAssetReloadFrameApplyReport {
        if self.lifecycle_interruption.is_some() {
            return self.interrupted_frame_report();
        }
        self.drain_runtime_frame_wake_token();
        let mut drain = self.drain_events(scheduler);
        let mut apply = self.commit_staged_into_world(world);
        apply.failed.append(&mut drain.failed);
        if apply.lifecycle_interruption().is_none() {
            let ready = self.collect_ready_report();
            self.stage_ready_for_world(scheduler, world, ready, &mut apply);
        }
        apply.pending_count = self.pending_count();
        if apply.apply_budget_overrun {
            self.diagnostics.apply_budget_overruns =
                self.diagnostics.apply_budget_overruns.saturating_add(1);
        }
        self.refresh_depth_diagnostics();
        DynamicSceneAssetReloadFrameApplyReport { drain, apply }
    }

    pub fn tick_into_level(
        &mut self,
        scheduler: &JobScheduler,
        level: &LevelSystem,
    ) -> DynamicSceneAssetReloadFrameApplyReport {
        if self.lifecycle_interruption.is_some() {
            return self.interrupted_frame_report();
        }
        self.drain_runtime_frame_wake_token();
        let mut drain = self.drain_events(scheduler);
        let mut apply = self.commit_staged_into_level(level);
        apply.failed.append(&mut drain.failed);
        if apply.lifecycle_interruption().is_none() {
            let ready = self.collect_ready_report();
            self.stage_ready_for_level(scheduler, level, ready, &mut apply);
        }
        apply.pending_count = self.pending_count();
        if apply.apply_budget_overrun {
            self.diagnostics.apply_budget_overruns =
                self.diagnostics.apply_budget_overruns.saturating_add(1);
        }
        self.refresh_depth_diagnostics();
        DynamicSceneAssetReloadFrameApplyReport { drain, apply }
    }

    pub fn target_staging_count(&self) -> usize {
        self.target_staging.len()
    }

    fn active_worker_count(&self) -> usize {
        self.pending.len().saturating_add(self.target_staging.len())
    }

    fn target_staging_reserved_bytes(&self) -> usize {
        self.target_staging.values().fold(0usize, |bytes, task| {
            bytes.saturating_add(task.reserved_bytes())
        })
    }

    fn drain_runtime_frame_wake_token(&self) {
        let Some(token) = self.runtime_frame_wake_token.as_ref() else {
            return;
        };
        let _ = token.try_recv();
    }

    pub(crate) fn has_pending_work(&self) -> bool {
        if self.lifecycle_interruption.is_some() {
            // Retained terminal rows are not runnable work. Disposal owns their reclamation.
            return false;
        }
        if self.source_error.is_some() {
            // Cancelled physical workers retain their permits until settlement.
            return self.active_worker_count() > 0;
        }
        self.pending_count() > 0
            || self.carried_event.is_some()
            || self.reconciliation.is_some()
            || !self.events.is_empty()
    }
}

#[cfg(test)]
#[path = "queue/tests/registered_source.rs"]
mod registered_source_tests;

fn locator_metadata_bytes(locator: &AssetUri) -> usize {
    locator
        .path()
        .len()
        .saturating_add(locator.label().map(str::len).unwrap_or(0))
        .saturating_add(12)
}
