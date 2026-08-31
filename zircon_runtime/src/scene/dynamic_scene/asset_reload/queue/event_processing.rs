use std::time::Instant;

use crate::{
    asset::{AssetEvent, AssetEventKind, AssetId, SceneAsset, facade::AssetEventPoll},
    core::{JobScheduler, TaskState},
    scene::dynamic_scene::DynamicSceneSpawnTask,
};

use super::{
    super::{
        reports::DynamicSceneAssetReloadDrainReport,
        result::DynamicSceneAssetReloadSupersededTask,
        skip::{DynamicSceneAssetReloadSkipReason, DynamicSceneAssetReloadSkippedEvent},
        task::DynamicSceneAssetReloadTask,
    },
    DeferredReload, DynamicSceneAssetReloadQueue, LATEST_REVISION_METADATA_BYTES,
    LatestRevisionState, ORDER_ENTRY_METADATA_BYTES, locator_metadata_bytes,
};

impl DynamicSceneAssetReloadQueue {
    pub fn drain_events(&mut self, scheduler: &JobScheduler) -> DynamicSceneAssetReloadDrainReport {
        let started = Instant::now();
        let mut report = DynamicSceneAssetReloadDrainReport::default();
        self.refresh_project_catalog_generation(&mut report);
        self.prune_expired_latest_revisions();
        self.retire_superseded_tasks();
        self.schedule_deferred(scheduler, &mut report);

        let mut allow_live_events = true;
        if let Some(poll) = self.carried_event.take() {
            allow_live_events = self.process_event_poll(scheduler, poll, &mut report);
        }
        if allow_live_events && self.reconciliation.is_some() {
            self.drain_reconciliation(scheduler, &mut report, started);
            allow_live_events = self.reconciliation.is_none() && self.carried_event.is_none();
        }

        while allow_live_events && self.event_budget_available(&mut report, started) {
            let poll = match self.events.try_recv_one() {
                Ok(poll) => poll,
                Err(crate::core::resource::ResourceEventTryRecvError::Empty) => break,
                Err(crate::core::resource::ResourceEventTryRecvError::Lagged(gap)) => {
                    report.generation_gap = Some(gap);
                    self.cancel_all_for_generation_gap(&mut report);
                    self.begin_reconciliation();
                    break;
                }
                Err(crate::core::resource::ResourceEventTryRecvError::SequenceExhausted) => {
                    report.event_sequence_exhausted = true;
                    break;
                }
                Err(crate::core::resource::ResourceEventTryRecvError::Disconnected) => {
                    report.receiver_disconnected = true;
                    break;
                }
            };

            if !self.process_event_poll(scheduler, poll, &mut report) {
                break;
            }
        }

        report.elapsed = started.elapsed();
        report.pending_count = self.pending_count();
        report.pending_metadata_bytes = self.pending_metadata_bytes;
        self.record_drain_diagnostics(&report);
        self.refresh_depth_diagnostics();
        report
    }

    pub(super) fn event_budget_available(
        &self,
        report: &mut DynamicSceneAssetReloadDrainReport,
        started: Instant,
    ) -> bool {
        if report.raw_events_examined >= self.limits.max_events_per_tick
            || report.event_bytes_drained >= self.limits.max_event_bytes_per_tick
            || (report.raw_events_examined > 0
                && started.elapsed() >= self.limits.event_time_budget)
        {
            report.event_budget_exhausted = true;
            return false;
        }
        true
    }

    pub(super) fn process_event_poll(
        &mut self,
        scheduler: &JobScheduler,
        poll: AssetEventPoll<SceneAsset>,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) -> bool {
        let poll_bytes = poll.approximate_bytes();
        let projected_bytes = report.event_bytes_drained.saturating_add(poll_bytes);
        if projected_bytes > self.limits.max_event_bytes_per_tick {
            report.event_budget_exhausted = true;
            if poll_bytes <= self.limits.max_event_bytes_per_tick {
                self.carried_event = Some(poll);
                return false;
            }
            report.raw_events_examined = report.raw_events_examined.saturating_add(1);
            match poll {
                AssetEventPoll::Relevant { event, .. } => self.drop_for_capacity(event, report),
                AssetEventPoll::Filtered { .. } => {
                    report.filtered_events = report.filtered_events.saturating_add(1);
                }
            }
            return false;
        }

        report.raw_events_examined = report.raw_events_examined.saturating_add(1);
        report.event_bytes_drained = projected_bytes;
        match poll {
            AssetEventPoll::Relevant { event, .. } => {
                report.events_drained = report.events_drained.saturating_add(1);
                self.handle_event(scheduler, event, report);
            }
            AssetEventPoll::Filtered { .. } => {
                report.filtered_events = report.filtered_events.saturating_add(1);
            }
        }
        true
    }

    pub(super) fn handle_event(
        &mut self,
        scheduler: &JobScheduler,
        event: AssetEvent<SceneAsset>,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        let asset_id = event.handle().id();
        let revision = event.revision();
        let authority_rank = event_authority_rank(event.event_kind());
        if self.latest_revisions.get(&asset_id).is_some_and(|latest| {
            revision < latest.revision
                || (revision == latest.revision && authority_rank <= latest.authority_rank)
        }) {
            report
                .skipped
                .push(DynamicSceneAssetReloadSkippedEvent::new(
                    event,
                    DynamicSceneAssetReloadSkipReason::StaleRevision,
                ));
            return;
        }

        if !self.record_latest_revision(asset_id, revision, authority_rank) {
            self.drop_for_capacity(event, report);
            return;
        }
        self.supersede_asset(asset_id, revision, report);

        match event.event_kind() {
            AssetEventKind::Removed | AssetEventKind::ReloadFailed => {
                let reason = if event.event_kind() == AssetEventKind::Removed {
                    self.remove_latest_revision_state(asset_id);
                    DynamicSceneAssetReloadSkipReason::Removed
                } else {
                    DynamicSceneAssetReloadSkipReason::ReloadFailed
                };
                report
                    .skipped
                    .push(DynamicSceneAssetReloadSkippedEvent::new(event, reason));
            }
            AssetEventKind::Added | AssetEventKind::Modified | AssetEventKind::Renamed => {
                let Some(uri) = event.locator().cloned() else {
                    report
                        .skipped
                        .push(DynamicSceneAssetReloadSkippedEvent::new(
                            event,
                            DynamicSceneAssetReloadSkipReason::MissingLocator,
                        ));
                    return;
                };
                let label = scene_asset_reload_label(&event, &uri);
                let metadata_bytes = estimate_task_metadata_bytes(&event, &label);
                let deferred = DeferredReload {
                    event,
                    uri,
                    label,
                    metadata_bytes,
                    queued_at: Instant::now(),
                };
                if self.pending.contains_key(&asset_id)
                    || self.target_staging.contains_key(&asset_id)
                    || self.active_worker_count() >= self.limits.max_active_tasks
                    || report.scheduled >= self.limits.max_schedules_per_tick
                {
                    self.defer_reload(asset_id, deferred, report);
                } else if self
                    .can_add_metadata(metadata_bytes.saturating_add(ORDER_ENTRY_METADATA_BYTES))
                {
                    self.schedule_reload(scheduler, asset_id, deferred, report);
                } else {
                    self.drop_for_capacity(deferred.event, report);
                }
            }
        }
    }

    fn schedule_reload(
        &mut self,
        scheduler: &JobScheduler,
        asset_id: AssetId,
        deferred: DeferredReload,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        let task = match self.task_graph_scope.as_ref() {
            Some(scope) => DynamicSceneSpawnTask::schedule_scene_asset_uri_with_limit_in_scope(
                scope,
                scheduler,
                self.project.clone(),
                deferred.uri,
                deferred.label.clone(),
                self.limits.max_prepared_scene_bytes,
            )
            .unwrap_or_else(|error| {
                DynamicSceneSpawnTask::rejected(deferred.label, error.to_string())
            }),
            None => DynamicSceneSpawnTask::schedule_scene_asset_uri_with_limit(
                scheduler,
                self.project.clone(),
                deferred.uri,
                deferred.label,
                self.limits.max_prepared_scene_bytes,
            ),
        };
        self.pending_metadata_bytes = self
            .pending_metadata_bytes
            .saturating_add(deferred.metadata_bytes);
        if self.pending_order.push_back(asset_id) {
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_add(ORDER_ENTRY_METADATA_BYTES);
        }
        self.pending.insert(
            asset_id,
            DynamicSceneAssetReloadTask::new(deferred.event, task, deferred.metadata_bytes),
        );
        report.scheduled = report.scheduled.saturating_add(1);
    }

    fn defer_reload(
        &mut self,
        asset_id: AssetId,
        deferred: DeferredReload,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        let prior_bytes = self
            .deferred
            .get(&asset_id)
            .map_or(0, |prior| prior.metadata_bytes);
        let order_bytes = usize::from(!self.deferred_order.contains(asset_id))
            .saturating_mul(ORDER_ENTRY_METADATA_BYTES);
        let projected = self
            .pending_metadata_bytes
            .saturating_sub(prior_bytes)
            .saturating_add(deferred.metadata_bytes)
            .saturating_add(order_bytes);
        if projected > self.limits.max_pending_metadata_bytes {
            self.remove_deferred(asset_id);
            self.drop_for_capacity(deferred.event, report);
            return;
        }
        self.deferred_order.push_back(asset_id);
        self.pending_metadata_bytes = projected;
        self.deferred.insert(asset_id, deferred);
        if report.scheduled >= self.limits.max_schedules_per_tick {
            report.schedule_budget_exhausted = true;
        }
    }

    fn schedule_deferred(
        &mut self,
        scheduler: &JobScheduler,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        let attempts = self.deferred_order.len();
        for _ in 0..attempts {
            if self.active_worker_count() >= self.limits.max_active_tasks
                || report.scheduled >= self.limits.max_schedules_per_tick
            {
                report.schedule_budget_exhausted = !self.deferred.is_empty();
                break;
            }
            let Some(asset_id) = self.deferred_order.pop_front() else {
                break;
            };
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_sub(ORDER_ENTRY_METADATA_BYTES);
            if self.pending.contains_key(&asset_id) || self.target_staging.contains_key(&asset_id) {
                if self.deferred_order.push_back(asset_id) {
                    self.pending_metadata_bytes = self
                        .pending_metadata_bytes
                        .saturating_add(ORDER_ENTRY_METADATA_BYTES);
                }
                continue;
            }
            let Some(deferred) = self.deferred.remove(&asset_id) else {
                continue;
            };
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_sub(deferred.metadata_bytes);
            if self.is_latest_event(&deferred.event) {
                self.schedule_reload(scheduler, asset_id, deferred, report);
            }
        }
    }

    fn supersede_asset(
        &mut self,
        asset_id: AssetId,
        latest_revision: u64,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        self.remove_deferred(asset_id);
        if let Some(previous) = self.ready.remove(&asset_id) {
            self.ready_result_bytes = self
                .ready_result_bytes
                .saturating_sub(previous.estimated_bytes());
            self.diagnostics.wasted_completed_tasks =
                self.diagnostics.wasted_completed_tasks.saturating_add(1);
            report
                .superseded_pending
                .push(DynamicSceneAssetReloadSupersededTask::new(
                    previous.event().clone(),
                    latest_revision,
                    false,
                    TaskState::Completed,
                ));
        }

        if let Some(task) = self.target_staging.get(&asset_id) {
            let previous_event = task.event().clone();
            let previous_state = task.state();
            let cancellation_requested = task.request_cancel();
            let worker_finished = task.is_ready();
            if cancellation_requested {
                self.diagnostics.cancellation_requests =
                    self.diagnostics.cancellation_requests.saturating_add(1);
                if !worker_finished {
                    self.diagnostics.cancelled_running_tasks =
                        self.diagnostics.cancelled_running_tasks.saturating_add(1);
                }
            }
            if worker_finished {
                self.remove_target_staging_without_order(asset_id);
                self.diagnostics.wasted_completed_tasks =
                    self.diagnostics.wasted_completed_tasks.saturating_add(1);
            }
            report
                .superseded_pending
                .push(DynamicSceneAssetReloadSupersededTask::new(
                    previous_event,
                    latest_revision,
                    cancellation_requested,
                    previous_state,
                ));
        }

        let Some(task) = self.pending.get(&asset_id) else {
            return;
        };
        let previous_event = task.event.clone();
        let previous_state = task.task.status_snapshot().state;
        let cancellation_requested = task.task.request_cancel();
        let worker_finished = task.task.is_ready();
        if cancellation_requested {
            self.diagnostics.cancellation_requests =
                self.diagnostics.cancellation_requests.saturating_add(1);
        }
        if !worker_finished {
            if cancellation_requested {
                self.diagnostics.cancelled_running_tasks =
                    self.diagnostics.cancelled_running_tasks.saturating_add(1);
            }
        } else {
            self.remove_pending(asset_id);
            self.diagnostics.wasted_completed_tasks =
                self.diagnostics.wasted_completed_tasks.saturating_add(1);
        }
        report
            .superseded_pending
            .push(DynamicSceneAssetReloadSupersededTask::new(
                previous_event,
                latest_revision,
                cancellation_requested,
                previous_state,
            ));
    }

    fn retire_superseded_tasks(&mut self) {
        let retired = self
            .pending
            .iter()
            .filter_map(|(asset_id, task)| {
                (task.task.is_ready() && !self.is_latest_event(&task.event)).then_some(*asset_id)
            })
            .collect::<Vec<_>>();
        for asset_id in retired {
            if let Some(task) = self.remove_pending(asset_id) {
                let _ = task.task.take_ready();
                self.diagnostics.stale_results = self.diagnostics.stale_results.saturating_add(1);
                self.diagnostics.wasted_completed_tasks =
                    self.diagnostics.wasted_completed_tasks.saturating_add(1);
            }
        }
        let retired_stages = self
            .target_staging
            .iter()
            .filter_map(|(asset_id, task)| {
                (task.is_ready() && !self.is_latest_event(task.event())).then_some(*asset_id)
            })
            .collect::<Vec<_>>();
        for asset_id in retired_stages {
            if let Some(task) = self.remove_target_staging_without_order(asset_id) {
                let _ = task.take_ready();
                self.diagnostics.stale_results = self.diagnostics.stale_results.saturating_add(1);
                self.diagnostics.wasted_completed_tasks =
                    self.diagnostics.wasted_completed_tasks.saturating_add(1);
            }
        }
    }

    fn cancel_all_for_generation_gap(&mut self, report: &mut DynamicSceneAssetReloadDrainReport) {
        self.clear_deferred();
        for task in self.pending.values() {
            let previous_state = task.task.status_snapshot().state;
            let cancellation_requested = task.task.request_cancel();
            if cancellation_requested {
                self.diagnostics.cancellation_requests =
                    self.diagnostics.cancellation_requests.saturating_add(1);
            }
            if cancellation_requested && !task.task.is_ready() {
                self.diagnostics.cancelled_running_tasks =
                    self.diagnostics.cancelled_running_tasks.saturating_add(1);
            }
            report
                .superseded_pending
                .push(DynamicSceneAssetReloadSupersededTask::new(
                    task.event.clone(),
                    u64::MAX,
                    cancellation_requested,
                    previous_state,
                ));
        }
        for task in self.target_staging.values() {
            let previous_state = task.state();
            let cancellation_requested = task.request_cancel();
            if cancellation_requested {
                self.diagnostics.cancellation_requests =
                    self.diagnostics.cancellation_requests.saturating_add(1);
                if !task.is_ready() {
                    self.diagnostics.cancelled_running_tasks =
                        self.diagnostics.cancelled_running_tasks.saturating_add(1);
                }
            }
            report
                .superseded_pending
                .push(DynamicSceneAssetReloadSupersededTask::new(
                    task.event().clone(),
                    u64::MAX,
                    cancellation_requested,
                    previous_state,
                ));
        }
        self.diagnostics.wasted_completed_tasks = self
            .diagnostics
            .wasted_completed_tasks
            .saturating_add(self.ready.len() as u64);
        self.ready.clear();
        self.pending_metadata_bytes = self.pending_metadata_bytes.saturating_sub(
            self.ready_order
                .len()
                .saturating_mul(ORDER_ENTRY_METADATA_BYTES),
        );
        self.ready_order.clear();
        self.ready_result_bytes = 0;
        self.pending_metadata_bytes = self.pending_metadata_bytes.saturating_sub(
            self.latest_revisions
                .len()
                .saturating_mul(LATEST_REVISION_METADATA_BYTES),
        );
        self.pending_metadata_bytes = self.pending_metadata_bytes.saturating_sub(
            self.latest_order
                .len()
                .saturating_mul(ORDER_ENTRY_METADATA_BYTES),
        );
        self.latest_revisions.clear();
        self.latest_order.clear();
    }

    fn record_latest_revision(
        &mut self,
        asset_id: AssetId,
        revision: u64,
        authority_rank: u8,
    ) -> bool {
        if let Some(latest) = self.latest_revisions.get_mut(&asset_id) {
            latest.revision = revision;
            latest.authority_rank = authority_rank;
            latest.observed_at = Instant::now();
            return true;
        }
        let required_bytes = LATEST_REVISION_METADATA_BYTES.saturating_add(
            usize::from(!self.latest_order.contains(asset_id))
                .saturating_mul(ORDER_ENTRY_METADATA_BYTES),
        );
        if self.latest_revisions.len() >= self.limits.max_latest_entries
            || !self.can_add_metadata(required_bytes)
        {
            return false;
        }
        self.latest_revisions.insert(
            asset_id,
            LatestRevisionState {
                revision,
                authority_rank,
                observed_at: Instant::now(),
            },
        );
        self.pending_metadata_bytes = self
            .pending_metadata_bytes
            .saturating_add(LATEST_REVISION_METADATA_BYTES);
        if self.latest_order.push_back(asset_id) {
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_add(ORDER_ENTRY_METADATA_BYTES);
        }
        true
    }

    fn remove_latest_revision_state(&mut self, asset_id: AssetId) {
        if self.latest_revisions.remove(&asset_id).is_some() {
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_sub(LATEST_REVISION_METADATA_BYTES);
        }
    }

    fn prune_expired_latest_revisions(&mut self) {
        let now = Instant::now();
        let ttl = self.limits.latest_revision_ttl;
        let inspections = self
            .latest_order
            .len()
            .min(self.limits.max_events_per_tick.max(1));
        for _ in 0..inspections {
            let Some(asset_id) = self.latest_order.pop_front() else {
                break;
            };
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_sub(ORDER_ENTRY_METADATA_BYTES);
            let expired = self.latest_revisions.get(&asset_id).is_some_and(|state| {
                !self.pending.contains_key(&asset_id)
                    && !self.target_staging.contains_key(&asset_id)
                    && !self.deferred.contains_key(&asset_id)
                    && !self.ready.contains_key(&asset_id)
                    && now.saturating_duration_since(state.observed_at) >= ttl
            });
            if expired {
                self.latest_revisions.remove(&asset_id);
                self.pending_metadata_bytes = self
                    .pending_metadata_bytes
                    .saturating_sub(LATEST_REVISION_METADATA_BYTES);
            } else if self.latest_revisions.contains_key(&asset_id) {
                if self.latest_order.push_back(asset_id) {
                    self.pending_metadata_bytes = self
                        .pending_metadata_bytes
                        .saturating_add(ORDER_ENTRY_METADATA_BYTES);
                }
            }
        }
    }

    fn record_drain_diagnostics(&mut self, report: &DynamicSceneAssetReloadDrainReport) {
        self.diagnostics.events_drained = self
            .diagnostics
            .events_drained
            .saturating_add(report.events_drained as u64);
        self.diagnostics.raw_events_examined = self
            .diagnostics
            .raw_events_examined
            .saturating_add(report.raw_events_examined as u64);
        self.diagnostics.filtered_events = self
            .diagnostics
            .filtered_events
            .saturating_add(report.filtered_events as u64);
        self.diagnostics.event_bytes_drained = self
            .diagnostics
            .event_bytes_drained
            .saturating_add(report.event_bytes_drained as u64);
        self.diagnostics.tasks_scheduled = self
            .diagnostics
            .tasks_scheduled
            .saturating_add(report.scheduled as u64);
        if report.generation_gap.is_some() {
            self.diagnostics.generation_gaps = self.diagnostics.generation_gaps.saturating_add(1);
        }
        if report.elapsed > self.limits.event_time_budget {
            self.diagnostics.event_budget_overruns =
                self.diagnostics.event_budget_overruns.saturating_add(1);
        }
    }

    fn drop_for_capacity(
        &mut self,
        event: AssetEvent<SceneAsset>,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        self.diagnostics.dropped_events = self.diagnostics.dropped_events.saturating_add(1);
        report
            .skipped
            .push(DynamicSceneAssetReloadSkippedEvent::new(
                event,
                DynamicSceneAssetReloadSkipReason::CapacityExceeded,
            ));
    }

    pub(super) fn can_add_metadata(&self, additional_bytes: usize) -> bool {
        self.pending_metadata_bytes.saturating_add(additional_bytes)
            <= self.limits.max_pending_metadata_bytes
    }

    pub(super) fn is_latest_event(&self, event: &AssetEvent<SceneAsset>) -> bool {
        self.latest_revisions
            .get(&event.handle().id())
            .is_some_and(|latest| {
                latest.revision == event.revision()
                    && latest.authority_rank == event_authority_rank(event.event_kind())
                    && event_is_loadable(event.event_kind())
            })
    }

    fn refresh_project_catalog_generation(
        &mut self,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        let project = match &self.project_source {
            super::DynamicSceneReloadProjectSource::AssetManager(asset_manager) => {
                let Some(project) = asset_manager.current_project_manager() else {
                    return;
                };
                project
            }
            #[cfg(test)]
            super::DynamicSceneReloadProjectSource::Static => return,
        };
        let sequence = project.catalog_input_generation().sequence();
        if sequence == self.catalog_generation_sequence {
            return;
        }
        self.project = project;
        self.catalog_generation_sequence = sequence;
        self.cancel_all_for_generation_gap(report);
        self.begin_reconciliation();
    }

    fn remove_deferred(&mut self, asset_id: AssetId) -> Option<DeferredReload> {
        let removed = self.deferred.remove(&asset_id)?;
        self.pending_metadata_bytes = self
            .pending_metadata_bytes
            .saturating_sub(removed.metadata_bytes);
        Some(removed)
    }

    fn clear_deferred(&mut self) {
        let removed_bytes = self
            .deferred
            .values()
            .fold(0usize, |bytes, deferred| {
                bytes.saturating_add(deferred.metadata_bytes)
            })
            .saturating_add(
                self.deferred_order
                    .len()
                    .saturating_mul(ORDER_ENTRY_METADATA_BYTES),
            );
        self.deferred.clear();
        self.deferred_order.clear();
        self.pending_metadata_bytes = self.pending_metadata_bytes.saturating_sub(removed_bytes);
    }
}

pub(super) fn event_authority_rank(kind: AssetEventKind) -> u8 {
    match kind {
        AssetEventKind::Added | AssetEventKind::Modified => 0,
        AssetEventKind::Renamed => 1,
        AssetEventKind::ReloadFailed => 2,
        AssetEventKind::Removed => 3,
    }
}

fn event_is_loadable(kind: AssetEventKind) -> bool {
    matches!(
        kind,
        AssetEventKind::Added | AssetEventKind::Modified | AssetEventKind::Renamed
    )
}

fn estimate_task_metadata_bytes(event: &AssetEvent<SceneAsset>, label: &str) -> usize {
    std::mem::size_of::<DynamicSceneAssetReloadTask>()
        .saturating_add(label.len())
        .saturating_add(event.locator().map_or(0, locator_metadata_bytes))
        .saturating_add(event.previous_locator().map_or(0, locator_metadata_bytes))
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
