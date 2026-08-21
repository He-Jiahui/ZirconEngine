use std::time::Instant;

use crate::{
    core::JobScheduler,
    scene::{
        DynamicSceneError, LevelSystem,
        dynamic_scene::{PreparedDynamicSceneSpawn, StagedDynamicSceneSpawn},
    },
};

#[cfg(test)]
use crate::scene::World;

use super::{
    super::{
        reports::{DynamicSceneAssetReloadApplyReport, DynamicSceneAssetReloadReadyReport},
        result::{
            DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure,
            DynamicSceneAssetReloadStaleResult,
        },
        stage_task::{DynamicSceneAssetReloadStageTask, estimate_stage_task_metadata_bytes},
    },
    DynamicSceneAssetReloadQueue, ORDER_ENTRY_METADATA_BYTES,
};

impl DynamicSceneAssetReloadQueue {
    #[cfg(test)]
    pub(super) fn stage_ready_for_world(
        &mut self,
        scheduler: &JobScheduler,
        world: &mut World,
        ready: DynamicSceneAssetReloadReadyReport,
        apply: &mut DynamicSceneAssetReloadApplyReport,
    ) {
        self.stage_ready_with_schedule(
            scheduler,
            ready,
            apply,
            |scheduler, event, prepared, limit| {
                let started = Instant::now();
                let target = prepared.capture_world_target(world, limit)?;
                let elapsed = started.elapsed();
                Ok((
                    DynamicSceneAssetReloadStageTask::schedule(scheduler, event, prepared, target),
                    elapsed,
                ))
            },
        );
    }

    pub(super) fn stage_ready_for_level(
        &mut self,
        scheduler: &JobScheduler,
        level: &LevelSystem,
        ready: DynamicSceneAssetReloadReadyReport,
        apply: &mut DynamicSceneAssetReloadApplyReport,
    ) {
        self.stage_ready_with_schedule(
            scheduler,
            ready,
            apply,
            |scheduler, event, prepared, limit| {
                Ok((
                    DynamicSceneAssetReloadStageTask::schedule_for_level(
                        scheduler,
                        event,
                        prepared,
                        level.clone(),
                        limit,
                    ),
                    std::time::Duration::ZERO,
                ))
            },
        );
    }

    fn stage_ready_with_schedule(
        &mut self,
        scheduler: &JobScheduler,
        ready: DynamicSceneAssetReloadReadyReport,
        apply: &mut DynamicSceneAssetReloadApplyReport,
        mut schedule: impl FnMut(
            &JobScheduler,
            crate::asset::AssetEvent<crate::asset::SceneAsset>,
            PreparedDynamicSceneSpawn,
            usize,
        ) -> Result<
            (DynamicSceneAssetReloadStageTask, std::time::Duration),
            DynamicSceneError,
        >,
    ) {
        apply.stale.extend(ready.stale);
        let started = Instant::now();
        let mut synchronous_capture_elapsed = std::time::Duration::ZERO;
        let mut deferred = Vec::new();
        let mut unconsumed_ready_bytes = ready.ready.iter().fold(0usize, |bytes, result| {
            bytes.saturating_add(result.estimated_bytes())
        });

        for result in ready.ready {
            let result_bytes = result.estimated_bytes();
            if apply.target_stages_scheduled >= self.limits.max_apply_per_tick
                || self.active_worker_count() >= self.limits.max_active_tasks
                || (apply.target_stages_scheduled > 0
                    && started.elapsed() >= self.limits.apply_time_budget)
            {
                deferred.push(result);
                continue;
            }

            let event = result.event().clone();
            let prepared = match result.result() {
                Ok(prepared) => prepared,
                Err(_) => {
                    let (event, result) = result.into_parts();
                    unconsumed_ready_bytes = unconsumed_ready_bytes.saturating_sub(result_bytes);
                    apply.failed.push(DynamicSceneAssetReloadApplyFailure::new(
                        event,
                        result.expect_err("error result inspected above"),
                    ));
                    continue;
                }
            };
            let prepared_bytes = prepared.estimated_bytes();
            let other_ready_bytes = unconsumed_ready_bytes.saturating_sub(result_bytes);
            let remaining_bytes = self
                .limits
                .max_pending_result_bytes
                .saturating_sub(self.target_staging_reserved_bytes())
                .saturating_sub(other_ready_bytes)
                .saturating_sub(prepared_bytes);
            let target_limit = remaining_bytes.min(
                self.limits
                    .max_apply_bytes_per_tick
                    .saturating_sub(prepared_bytes),
            );
            let metadata_bytes = estimate_stage_task_metadata_bytes(&event);
            if target_limit == 0
                || !self.can_add_metadata(metadata_bytes.saturating_add(ORDER_ENTRY_METADATA_BYTES))
            {
                deferred.push(result);
                continue;
            }

            let (event, result) = result.into_parts();
            let prepared = result.expect("prepared result inspected above");
            let (task, capture_elapsed) =
                match schedule(scheduler, event.clone(), prepared, target_limit) {
                    Ok(scheduled) => scheduled,
                    Err(error) => {
                        unconsumed_ready_bytes =
                            unconsumed_ready_bytes.saturating_sub(result_bytes);
                        apply
                            .failed
                            .push(DynamicSceneAssetReloadApplyFailure::new(event, error));
                        continue;
                    }
                };
            synchronous_capture_elapsed = synchronous_capture_elapsed.max(capture_elapsed);
            let asset_id = event.handle().id();
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_add(task.estimated_metadata_bytes());
            if self.target_staging_order.push_back(asset_id) {
                self.pending_metadata_bytes = self
                    .pending_metadata_bytes
                    .saturating_add(ORDER_ENTRY_METADATA_BYTES);
            }
            self.target_staging.insert(asset_id, task);
            unconsumed_ready_bytes = unconsumed_ready_bytes.saturating_sub(result_bytes);
            apply.target_stages_scheduled = apply.target_stages_scheduled.saturating_add(1);
        }

        self.requeue_ready(deferred);
        apply.target_capture_elapsed = synchronous_capture_elapsed;
        apply.deferred_count = self.target_staging.len();
        apply.apply_budget_exhausted |= !self.ready.is_empty();
        apply.apply_budget_overrun |= started.elapsed() > self.limits.apply_time_budget;
        self.diagnostics.max_target_capture_duration = self
            .diagnostics
            .max_target_capture_duration
            .max(apply.target_capture_elapsed);
    }

    #[cfg(test)]
    pub(super) fn commit_staged_into_world(
        &mut self,
        world: &mut World,
    ) -> DynamicSceneAssetReloadApplyReport {
        self.commit_staged_with(|staged| staged.commit_into(world))
    }

    pub(super) fn commit_staged_into_level(
        &mut self,
        level: &LevelSystem,
    ) -> DynamicSceneAssetReloadApplyReport {
        self.commit_staged_with(|staged| staged.commit_into_level(level))
    }

    fn commit_staged_with(
        &mut self,
        mut commit: impl FnMut(
            StagedDynamicSceneSpawn,
        ) -> Result<crate::scene::EntityRemap, DynamicSceneError>,
    ) -> DynamicSceneAssetReloadApplyReport {
        let started = Instant::now();
        let mut report = DynamicSceneAssetReloadApplyReport::default();
        let attempts = self.target_staging_order.len();

        for _ in 0..attempts {
            let completed = report.applied.len().saturating_add(report.failed.len());
            if completed >= self.limits.max_apply_per_tick
                || (completed > 0 && started.elapsed() >= self.limits.apply_time_budget)
            {
                report.apply_budget_exhausted = !self.target_staging.is_empty();
                break;
            }
            let Some(asset_id) = self.target_staging_order.pop_front() else {
                break;
            };
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_sub(ORDER_ENTRY_METADATA_BYTES);
            let Some(task) = self.target_staging.get(&asset_id) else {
                continue;
            };
            if !task.is_ready() {
                if self.target_staging_order.push_back(asset_id) {
                    self.pending_metadata_bytes = self
                        .pending_metadata_bytes
                        .saturating_add(ORDER_ENTRY_METADATA_BYTES);
                }
                continue;
            }

            let event = task.event().clone();
            if !self.is_latest_event(&event) || task.is_cancellation_requested() {
                let _ = self.remove_target_staging_without_order(asset_id);
                report.stale.push(DynamicSceneAssetReloadStaleResult::new(
                    event,
                    self.latest_revisions
                        .get(&asset_id)
                        .map_or(u64::MAX, |latest| latest.revision),
                ));
                self.diagnostics.stale_results = self.diagnostics.stale_results.saturating_add(1);
                continue;
            }
            let task_bytes = task.reserved_bytes();
            if report.applied_bytes.saturating_add(task_bytes)
                > self.limits.max_apply_bytes_per_tick
            {
                if self.target_staging_order.push_back(asset_id) {
                    self.pending_metadata_bytes = self
                        .pending_metadata_bytes
                        .saturating_add(ORDER_ENTRY_METADATA_BYTES);
                }
                report.apply_budget_exhausted = true;
                break;
            }
            let task = self
                .remove_target_staging_without_order(asset_id)
                .expect("target stage task inspected above");
            report.target_capture_elapsed = report
                .target_capture_elapsed
                .max(task.target_capture_elapsed());
            let staged = match task
                .take_ready()
                .expect("completed target stage has a result")
            {
                Ok(staged) => staged,
                Err(error) => {
                    report
                        .failed
                        .push(DynamicSceneAssetReloadApplyFailure::new(event, error));
                    continue;
                }
            };
            report.applied_bytes = report.applied_bytes.saturating_add(task_bytes);
            let component_type_count = staged.component_type_count();
            let entity_count = staged.entity_count();
            let resource_count = staged.resource_count();
            match commit(staged) {
                Ok(remap) => report
                    .applied
                    .push(DynamicSceneAssetReloadAppliedScene::new(
                        event,
                        remap,
                        component_type_count,
                        entity_count,
                        resource_count,
                    )),
                Err(error) => report
                    .failed
                    .push(DynamicSceneAssetReloadApplyFailure::new(event, error)),
            }
        }

        report.deferred_count = self.target_staging.len();
        report.elapsed = started.elapsed();
        report.apply_budget_overrun = report.elapsed > self.limits.apply_time_budget;
        self.diagnostics.max_target_capture_duration = self
            .diagnostics
            .max_target_capture_duration
            .max(report.target_capture_elapsed);
        report
    }

    pub(super) fn remove_target_staging_without_order(
        &mut self,
        asset_id: crate::asset::AssetId,
    ) -> Option<DynamicSceneAssetReloadStageTask> {
        let removed = self.target_staging.remove(&asset_id)?;
        self.pending_metadata_bytes = self
            .pending_metadata_bytes
            .saturating_sub(removed.estimated_metadata_bytes());
        Some(removed)
    }
}
