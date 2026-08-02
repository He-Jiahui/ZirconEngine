use std::time::Instant;

use crate::{asset::AssetId, scene::dynamic_scene::DynamicSceneError};

use super::{
    super::{
        reports::{DynamicSceneAssetReloadPendingReport, DynamicSceneAssetReloadReadyReport},
        result::{DynamicSceneAssetReloadResult, DynamicSceneAssetReloadStaleResult},
        task::DynamicSceneAssetReloadPendingTaskSnapshot,
    },
    DynamicSceneAssetReloadQueue, LATEST_REVISION_METADATA_BYTES, ORDER_ENTRY_METADATA_BYTES,
};

impl DynamicSceneAssetReloadQueue {
    #[cfg(test)]
    pub(crate) fn take_ready(&mut self) -> Vec<DynamicSceneAssetReloadResult> {
        self.collect_ready_report().into_ready()
    }

    #[cfg(test)]
    pub(crate) fn take_ready_report(&mut self) -> DynamicSceneAssetReloadReadyReport {
        self.collect_ready_report()
    }

    pub(super) fn collect_ready_report(&mut self) -> DynamicSceneAssetReloadReadyReport {
        let started = Instant::now();
        let mut report = DynamicSceneAssetReloadReadyReport::default();
        let inspections = self.pending_order.len();

        for _ in 0..inspections {
            if report.inspected_count > 0 && started.elapsed() >= self.limits.ready_time_budget {
                report.ready_budget_exhausted = true;
                break;
            }
            let Some(asset_id) = self.pending_order.pop_front() else {
                break;
            };
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_sub(ORDER_ENTRY_METADATA_BYTES);
            report.inspected_count = report.inspected_count.saturating_add(1);
            let Some(pending) = self.pending.get(&asset_id) else {
                continue;
            };
            if !pending.task.is_ready() {
                if self.pending_order.push_back(asset_id) {
                    self.pending_metadata_bytes = self
                        .pending_metadata_bytes
                        .saturating_add(ORDER_ENTRY_METADATA_BYTES);
                }
                continue;
            }
            let Some(task) = self.remove_pending_without_order(asset_id) else {
                continue;
            };
            let event = task.event;
            let result = task.task.take_ready().unwrap_or_else(|| {
                Err(DynamicSceneError::SpawnTaskResultUnavailable {
                    label: task.task.descriptor().label.clone(),
                })
            });
            let latest_revision = self
                .latest_revisions
                .get(&asset_id)
                .map(|latest| latest.revision);
            if latest_revision != Some(event.revision()) || task.task.is_cancellation_requested() {
                report.stale.push(DynamicSceneAssetReloadStaleResult::new(
                    event,
                    latest_revision.unwrap_or(u64::MAX),
                ));
                self.diagnostics.stale_results = self.diagnostics.stale_results.saturating_add(1);
                continue;
            }
            self.insert_ready(DynamicSceneAssetReloadResult::new(event, result));
        }

        let ready_attempts = self.ready_order.len();
        for _ in 0..ready_attempts {
            if report.ready.len() >= self.limits.max_ready_per_tick {
                report.ready_budget_exhausted = !self.ready.is_empty();
                break;
            }
            if !report.ready.is_empty() && started.elapsed() >= self.limits.ready_time_budget {
                report.ready_budget_exhausted = true;
                break;
            }
            let Some(asset_id) = self.ready_order.pop_front() else {
                break;
            };
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_sub(ORDER_ENTRY_METADATA_BYTES);
            let Some(result_bytes) = self
                .ready
                .get(&asset_id)
                .map(DynamicSceneAssetReloadResult::estimated_bytes)
            else {
                continue;
            };
            if report.collected_bytes.saturating_add(result_bytes)
                > self.limits.max_ready_bytes_per_tick
            {
                if self.ready_order.push_back(asset_id) {
                    self.pending_metadata_bytes = self
                        .pending_metadata_bytes
                        .saturating_add(ORDER_ENTRY_METADATA_BYTES);
                }
                report.ready_budget_exhausted = true;
                break;
            }
            let result = self
                .ready
                .remove(&asset_id)
                .expect("ready result inspected above");
            self.ready_result_bytes = self.ready_result_bytes.saturating_sub(result_bytes);
            report.collected_bytes = report.collected_bytes.saturating_add(result_bytes);
            report.ready.push(result);
        }

        report.pending_count = self.pending_count();
        report.elapsed = started.elapsed();
        report.ready_budget_overrun = report.elapsed > self.limits.ready_time_budget;
        if report.ready_budget_overrun {
            self.diagnostics.ready_budget_overruns =
                self.diagnostics.ready_budget_overruns.saturating_add(1);
        }
        self.refresh_depth_diagnostics();
        report
    }

    pub fn pending_tasks(
        &self,
    ) -> impl ExactSizeIterator<Item = &super::super::task::DynamicSceneAssetReloadTask> {
        self.pending.values()
    }

    pub fn pending_count(&self) -> usize {
        self.pending
            .len()
            .saturating_add(
                self.target_staging
                    .keys()
                    .filter(|asset_id| !self.pending.contains_key(asset_id))
                    .count(),
            )
            .saturating_add(
                self.deferred
                    .keys()
                    .filter(|asset_id| {
                        !self.pending.contains_key(asset_id)
                            && !self.target_staging.contains_key(asset_id)
                    })
                    .count(),
            )
            .saturating_add(
                self.ready
                    .keys()
                    .filter(|asset_id| {
                        !self.pending.contains_key(asset_id)
                            && !self.target_staging.contains_key(asset_id)
                            && !self.deferred.contains_key(asset_id)
                    })
                    .count(),
            )
    }

    pub fn pending_report(&self) -> DynamicSceneAssetReloadPendingReport {
        DynamicSceneAssetReloadPendingReport {
            pending: self
                .pending
                .values()
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

    pub(super) fn remove_pending(
        &mut self,
        asset_id: AssetId,
    ) -> Option<super::super::task::DynamicSceneAssetReloadTask> {
        self.remove_pending_without_order(asset_id)
    }

    fn remove_pending_without_order(
        &mut self,
        asset_id: AssetId,
    ) -> Option<super::super::task::DynamicSceneAssetReloadTask> {
        let removed = self.pending.remove(&asset_id)?;
        self.pending_metadata_bytes = self
            .pending_metadata_bytes
            .saturating_sub(removed.estimated_metadata_bytes());
        Some(removed)
    }

    pub(super) fn insert_ready(&mut self, result: DynamicSceneAssetReloadResult) {
        let limit_bytes = self
            .limits
            .max_pending_result_bytes
            .min(self.limits.max_ready_bytes_per_tick)
            .min(self.limits.max_apply_bytes_per_tick);
        let result = result.bounded_to(limit_bytes);
        let asset_id = result.event().handle().id();
        let result_bytes = result.estimated_bytes();
        let replaced_bytes = self
            .ready
            .get(&asset_id)
            .map_or(0, DynamicSceneAssetReloadResult::estimated_bytes);
        let projected_bytes = self
            .ready_result_bytes
            .saturating_sub(replaced_bytes)
            .saturating_add(self.target_staging_reserved_bytes)
            .saturating_add(result_bytes);
        if projected_bytes > self.limits.max_pending_result_bytes {
            self.diagnostics.dropped_events = self.diagnostics.dropped_events.saturating_add(1);
            return;
        }
        if let Some(replaced) = self.ready.remove(&asset_id) {
            self.ready_result_bytes = self
                .ready_result_bytes
                .saturating_sub(replaced.estimated_bytes());
        }
        if self.ready_order.push_back(asset_id) {
            self.pending_metadata_bytes = self
                .pending_metadata_bytes
                .saturating_add(ORDER_ENTRY_METADATA_BYTES);
        }
        self.ready_result_bytes = self.ready_result_bytes.saturating_add(result_bytes);
        self.ready.insert(asset_id, result);
    }

    pub(super) fn requeue_ready(&mut self, deferred: Vec<DynamicSceneAssetReloadResult>) {
        for result in deferred {
            if self
                .latest_revisions
                .get(&result.event().handle().id())
                .is_some_and(|latest| {
                    latest.revision == result.event().revision()
                        && latest.authority_rank
                            == super::event_processing::event_authority_rank(
                                result.event().event_kind(),
                            )
                })
            {
                self.insert_ready(result);
            } else {
                self.diagnostics.stale_results = self.diagnostics.stale_results.saturating_add(1);
            }
        }
    }

    pub(super) fn refresh_depth_diagnostics(&mut self) {
        self.diagnostics.active_tasks = self.active_worker_count();
        self.diagnostics.deferred_reloads = self.deferred.len();
        self.diagnostics.ready_results = self.ready.len();
        self.diagnostics.target_staging_tasks = self.target_staging.len();
        self.diagnostics.latest_entries = self.latest_revisions.len();
        self.diagnostics.pending_metadata_bytes = self.pending_metadata_bytes;
        self.diagnostics.ready_result_bytes = self.ready_result_bytes;
        self.diagnostics.target_staging_reserved_bytes = self.target_staging_reserved_bytes;
        self.diagnostics.resident_result_bytes = self
            .ready_result_bytes
            .saturating_add(self.target_staging_reserved_bytes);
        self.diagnostics.max_active_tasks = self
            .diagnostics
            .max_active_tasks
            .max(self.diagnostics.active_tasks);
        self.diagnostics.max_deferred_reloads = self
            .diagnostics
            .max_deferred_reloads
            .max(self.diagnostics.deferred_reloads);
        self.diagnostics.max_ready_results = self
            .diagnostics
            .max_ready_results
            .max(self.diagnostics.ready_results);
        self.diagnostics.max_target_staging_tasks = self
            .diagnostics
            .max_target_staging_tasks
            .max(self.diagnostics.target_staging_tasks);
        self.diagnostics.max_pending_metadata_bytes = self
            .diagnostics
            .max_pending_metadata_bytes
            .max(self.diagnostics.pending_metadata_bytes);
        self.diagnostics.max_ready_result_bytes = self
            .diagnostics
            .max_ready_result_bytes
            .max(self.diagnostics.ready_result_bytes);
        self.diagnostics.max_target_staging_reserved_bytes = self
            .diagnostics
            .max_target_staging_reserved_bytes
            .max(self.diagnostics.target_staging_reserved_bytes);
        self.diagnostics.max_resident_result_bytes = self
            .diagnostics
            .max_resident_result_bytes
            .max(self.diagnostics.resident_result_bytes);
        debug_assert!(
            self.diagnostics.resident_result_bytes <= self.limits.max_pending_result_bytes
        );
        debug_assert_eq!(
            self.pending_metadata_bytes,
            self.pending
                .values()
                .map(|task| task.estimated_metadata_bytes())
                .sum::<usize>()
                + self.pending_order.len() * ORDER_ENTRY_METADATA_BYTES
                + self
                    .deferred
                    .values()
                    .map(|reload| reload.metadata_bytes)
                    .sum::<usize>()
                + self.deferred_order.len() * ORDER_ENTRY_METADATA_BYTES
                + self.latest_revisions.len() * LATEST_REVISION_METADATA_BYTES
                + self.latest_order.len() * ORDER_ENTRY_METADATA_BYTES
                + self.ready_order.len() * ORDER_ENTRY_METADATA_BYTES
                + self
                    .target_staging
                    .values()
                    .map(|task| task.estimated_metadata_bytes())
                    .sum::<usize>()
                + self.target_staging_order.len() * ORDER_ENTRY_METADATA_BYTES
        );
    }
}
