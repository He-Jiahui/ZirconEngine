mod event_processing;
mod order;
mod ready_collection;
mod reconciliation;
mod target_staging;

use std::{collections::HashMap, time::Instant};

use crate::{
    asset::{
        AssetEvent, AssetEventReceiver, AssetId, AssetUri, ProjectAssetManager, SceneAsset,
        facade::AssetEventPoll, project::ProjectManager,
    },
    core::{
        JobScheduler, TaskGraphScope,
        framework::channel::{ChannelReceiver, ChannelWakeCallback},
        resource::ResourceManager,
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
    AssetManager(ProjectAssetManager),
    #[cfg(test)]
    Static,
}

pub struct DynamicSceneAssetReloadQueue {
    project: ProjectManager,
    project_source: DynamicSceneReloadProjectSource,
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
        asset_manager: &ProjectAssetManager,
    ) -> Self {
        Self::from_project_asset_manager_with_limits(
            project,
            asset_manager,
            DynamicSceneAssetReloadLimits::default(),
        )
    }

    pub fn from_project_asset_manager_with_limits(
        project: ProjectManager,
        asset_manager: &ProjectAssetManager,
        limits: DynamicSceneAssetReloadLimits,
    ) -> Self {
        Self::construct(
            project,
            DynamicSceneReloadProjectSource::AssetManager(asset_manager.clone()),
            asset_manager.subscribe_asset_events::<SceneAsset>(),
            asset_manager.resource_manager(),
            limits,
        )
    }

    pub(crate) fn with_task_graph_scope(mut self, task_graph_scope: TaskGraphScope) -> Self {
        self.task_graph_scope = Some(task_graph_scope);
        self
    }

    pub(crate) fn install_runtime_frame_wake(&mut self, wake: ChannelWakeCallback) {
        let DynamicSceneReloadProjectSource::AssetManager(asset_manager) = &self.project_source
        else {
            return;
        };
        self.runtime_frame_wake_token = Some(asset_manager.subscribe_project_generation_wake(wake));
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
        self.drain_runtime_frame_wake_token();
        let drain = self.drain_events(scheduler);
        let mut apply = self.commit_staged_into_world(world);
        let ready = self.collect_ready_report();
        self.stage_ready_for_world(scheduler, world, ready, &mut apply);
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
        self.drain_runtime_frame_wake_token();
        let drain = self.drain_events(scheduler);
        let mut apply = self.commit_staged_into_level(level);
        let ready = self.collect_ready_report();
        self.stage_ready_for_level(scheduler, level, ready, &mut apply);
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
        self.pending_count() > 0
            || self.carried_event.is_some()
            || self.reconciliation.is_some()
            || !self.events.is_empty()
    }
}

fn locator_metadata_bytes(locator: &AssetUri) -> usize {
    locator
        .path()
        .len()
        .saturating_add(locator.label().map(str::len).unwrap_or(0))
        .saturating_add(12)
}
