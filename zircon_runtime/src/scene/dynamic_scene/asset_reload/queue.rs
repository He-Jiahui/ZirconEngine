use std::collections::HashMap;

use crate::{
    asset::{
        project::ProjectManager, AssetEvent, AssetEventKind, AssetEventReceiver, AssetId,
        ProjectAssetManager, SceneAsset,
    },
    core::JobScheduler,
    scene::{dynamic_scene::DynamicSceneSpawnTask, LevelSystem, World},
};

use super::{
    reports::{
        DynamicSceneAssetReloadDrainReport, DynamicSceneAssetReloadFrameApplyReport,
        DynamicSceneAssetReloadPendingReport, DynamicSceneAssetReloadReadyReport,
        DynamicSceneAssetReloadTickReport,
    },
    result::{
        DynamicSceneAssetReloadResult, DynamicSceneAssetReloadStaleResult,
        DynamicSceneAssetReloadSupersededTask,
    },
    skip::{DynamicSceneAssetReloadSkipReason, DynamicSceneAssetReloadSkippedEvent},
    task::{DynamicSceneAssetReloadPendingTaskSnapshot, DynamicSceneAssetReloadTask},
};

pub struct DynamicSceneAssetReloadQueue {
    project: ProjectManager,
    events: AssetEventReceiver<SceneAsset>,
    pending: Vec<DynamicSceneAssetReloadTask>,
    /// Newer facade revisions make older pending preparations observable-but-discarded.
    latest_revisions: HashMap<AssetId, u64>,
}

impl DynamicSceneAssetReloadQueue {
    pub fn new(project: ProjectManager, events: AssetEventReceiver<SceneAsset>) -> Self {
        Self {
            project,
            events,
            pending: Vec::new(),
            latest_revisions: HashMap::new(),
        }
    }

    pub fn from_project_asset_manager(
        project: ProjectManager,
        asset_manager: &ProjectAssetManager,
    ) -> Self {
        Self::new(
            project,
            asset_manager.subscribe_asset_events::<SceneAsset>(),
        )
    }

    pub fn tick(&mut self, scheduler: &JobScheduler) -> DynamicSceneAssetReloadTickReport {
        let drain = self.drain_events(scheduler);
        let ready = self.take_ready_report();
        DynamicSceneAssetReloadTickReport { drain, ready }
    }

    pub fn tick_into(
        &mut self,
        scheduler: &JobScheduler,
        world: &mut World,
    ) -> DynamicSceneAssetReloadFrameApplyReport {
        let DynamicSceneAssetReloadTickReport { drain, ready } = self.tick(scheduler);
        let apply = ready.spawn_ready_into(world);
        DynamicSceneAssetReloadFrameApplyReport { drain, apply }
    }

    pub fn tick_into_level(
        &mut self,
        scheduler: &JobScheduler,
        level: &LevelSystem,
    ) -> DynamicSceneAssetReloadFrameApplyReport {
        let DynamicSceneAssetReloadTickReport { drain, ready } = self.tick(scheduler);
        let apply = ready.spawn_ready_into_level(level);
        DynamicSceneAssetReloadFrameApplyReport { drain, apply }
    }

    pub fn drain_events(&mut self, scheduler: &JobScheduler) -> DynamicSceneAssetReloadDrainReport {
        let mut report = DynamicSceneAssetReloadDrainReport::default();

        loop {
            let event = match self.events.try_recv() {
                Ok(event) => event,
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    report.receiver_disconnected = true;
                    break;
                }
            };

            report.events_drained += 1;
            let latest_revision = self.record_latest_revision(&event);
            report
                .superseded_pending
                .extend(self.take_superseded_pending(event.handle().id(), latest_revision));
            match event.event_kind() {
                AssetEventKind::Added | AssetEventKind::Modified | AssetEventKind::Renamed => {
                    if event.revision() < latest_revision {
                        report
                            .skipped
                            .push(DynamicSceneAssetReloadSkippedEvent::new(
                                event,
                                DynamicSceneAssetReloadSkipReason::StaleRevision,
                            ));
                        continue;
                    }

                    let Some(uri) = event.locator().cloned() else {
                        report
                            .skipped
                            .push(DynamicSceneAssetReloadSkippedEvent::new(
                                event,
                                DynamicSceneAssetReloadSkipReason::MissingLocator,
                            ));
                        continue;
                    };
                    let label = scene_asset_reload_label(&event, &uri);
                    let task = DynamicSceneSpawnTask::schedule_scene_asset_uri(
                        scheduler,
                        self.project.clone(),
                        uri,
                        label,
                    );
                    self.pending
                        .push(DynamicSceneAssetReloadTask { event, task });
                    report.scheduled += 1;
                }
                AssetEventKind::Removed => {
                    report
                        .skipped
                        .push(DynamicSceneAssetReloadSkippedEvent::new(
                            event,
                            DynamicSceneAssetReloadSkipReason::Removed,
                        ));
                }
                AssetEventKind::ReloadFailed => {
                    report
                        .skipped
                        .push(DynamicSceneAssetReloadSkippedEvent::new(
                            event,
                            DynamicSceneAssetReloadSkipReason::ReloadFailed,
                        ));
                }
            }
        }

        report.pending_count = self.pending.len();
        report
    }

    pub fn take_ready(&mut self) -> Vec<DynamicSceneAssetReloadResult> {
        self.take_ready_report().into_ready()
    }

    pub fn take_ready_report(&mut self) -> DynamicSceneAssetReloadReadyReport {
        let mut report = DynamicSceneAssetReloadReadyReport::default();
        let mut pending = Vec::with_capacity(self.pending.len());

        for task in self.pending.drain(..) {
            let DynamicSceneAssetReloadTask { event, task } = task;
            if let Some(result) = task.take_ready() {
                let latest_revision = self
                    .latest_revisions
                    .get(&event.handle().id())
                    .copied()
                    .unwrap_or(event.revision());
                if event.revision() < latest_revision {
                    report.stale.push(DynamicSceneAssetReloadStaleResult::new(
                        event,
                        latest_revision,
                    ));
                } else {
                    report
                        .ready
                        .push(DynamicSceneAssetReloadResult::new(event, result));
                }
            } else {
                pending.push(DynamicSceneAssetReloadTask { event, task });
            }
        }

        self.pending = pending;
        report.pending_count = self.pending.len();
        report
    }

    pub fn pending(&self) -> &[DynamicSceneAssetReloadTask] {
        &self.pending
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn pending_report(&self) -> DynamicSceneAssetReloadPendingReport {
        DynamicSceneAssetReloadPendingReport {
            pending: self
                .pending
                .iter()
                .map(|task| {
                    DynamicSceneAssetReloadPendingTaskSnapshot::new(
                        task.event.clone(),
                        task.task.descriptor().clone(),
                        task.task.status_snapshot(),
                    )
                })
                .collect(),
        }
    }

    fn record_latest_revision(&mut self, event: &AssetEvent<SceneAsset>) -> u64 {
        let revision = event.revision();
        self.latest_revisions
            .entry(event.handle().id())
            .and_modify(|latest| {
                if revision > *latest {
                    *latest = revision;
                }
            })
            .or_insert(revision);
        *self
            .latest_revisions
            .get(&event.handle().id())
            .expect("latest scene asset reload revision must be recorded")
    }

    fn take_superseded_pending(
        &mut self,
        asset_id: AssetId,
        latest_revision: u64,
    ) -> Vec<DynamicSceneAssetReloadSupersededTask> {
        let mut retained = Vec::with_capacity(self.pending.len());
        let mut superseded = Vec::new();

        for task in self.pending.drain(..) {
            let is_superseded =
                task.event.handle().id() == asset_id && task.event.revision() < latest_revision;
            if is_superseded {
                let DynamicSceneAssetReloadTask { event, task: _ } = task;
                superseded.push(DynamicSceneAssetReloadSupersededTask::new(
                    event,
                    latest_revision,
                ));
            } else {
                retained.push(task);
            }
        }

        self.pending = retained;
        superseded
    }
}

fn scene_asset_reload_label(
    event: &AssetEvent<SceneAsset>,
    uri: &crate::asset::AssetUri,
) -> String {
    format!(
        "dynamic-scene-asset-reload:{}:{}@{}",
        scene_asset_event_kind_label(event.event_kind()),
        uri,
        event.revision()
    )
}

fn scene_asset_event_kind_label(kind: AssetEventKind) -> &'static str {
    match kind {
        AssetEventKind::Added => "added",
        AssetEventKind::Modified => "modified",
        AssetEventKind::Renamed => "renamed",
        AssetEventKind::Removed => "removed",
        AssetEventKind::ReloadFailed => "reload-failed",
    }
}
