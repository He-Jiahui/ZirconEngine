use std::time::Duration;

#[cfg(test)]
use std::time::Instant;

use crate::core::TaskState;

#[cfg(test)]
use crate::scene::{LevelSystem, World};

use super::{
    result::{
        DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure,
        DynamicSceneAssetReloadResult, DynamicSceneAssetReloadStaleResult,
        DynamicSceneAssetReloadSupersededTask,
    },
    skip::{DynamicSceneAssetReloadSkipReason, DynamicSceneAssetReloadSkippedEvent},
    task::DynamicSceneAssetReloadPendingTaskSnapshot,
};

#[derive(Clone, Debug, Default, PartialEq)]
/// 事件接纳阶段的单帧结果；跳过事件、取消请求和代际缺口不能与应用失败混为一谈。
pub struct DynamicSceneAssetReloadDrainReport {
    pub events_drained: usize,
    pub raw_events_examined: usize,
    pub filtered_events: usize,
    pub event_bytes_drained: usize,
    pub scheduled: usize,
    pub failed: Vec<DynamicSceneAssetReloadApplyFailure>,
    pub skipped: Vec<DynamicSceneAssetReloadSkippedEvent>,
    pub superseded_pending: Vec<DynamicSceneAssetReloadSupersededTask>,
    pub receiver_disconnected: bool,
    pub event_sequence_exhausted: bool,
    pub generation_gap: Option<crate::core::resource::ResourceEventGap>,
    pub pending_count: usize,
    pub pending_metadata_bytes: usize,
    pub event_budget_exhausted: bool,
    pub schedule_budget_exhausted: bool,
    pub elapsed: Duration,
}

impl DynamicSceneAssetReloadDrainReport {
    pub fn skipped_count(&self) -> usize {
        self.skipped.len()
    }

    pub fn skipped_count_for(&self, reason: DynamicSceneAssetReloadSkipReason) -> usize {
        self.skipped
            .iter()
            .filter(|event| event.reason() == reason)
            .count()
    }

    pub fn superseded_pending_count(&self) -> usize {
        self.superseded_pending.len()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DynamicSceneAssetReloadPendingReport {
    pub pending: Vec<DynamicSceneAssetReloadPendingTaskSnapshot>,
}

impl DynamicSceneAssetReloadPendingReport {
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn count_for_state(&self, state: TaskState) -> usize {
        self.pending
            .iter()
            .filter(|task| task.state() == state)
            .count()
    }

    pub fn collectable_count(&self) -> usize {
        self.pending
            .iter()
            .filter(|task| task.is_collectable())
            .count()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// 目标世界应用阶段的单帧结果；失败与陈旧分开报告，延后结果仍由队列持有。
pub struct DynamicSceneAssetReloadApplyReport {
    pub applied: Vec<DynamicSceneAssetReloadAppliedScene>,
    pub failed: Vec<DynamicSceneAssetReloadApplyFailure>,
    pub stale: Vec<DynamicSceneAssetReloadStaleResult>,
    pub pending_count: usize,
    pub applied_bytes: usize,
    pub target_stages_scheduled: usize,
    pub target_capture_elapsed: Duration,
    pub deferred_count: usize,
    pub apply_budget_exhausted: bool,
    pub apply_budget_overrun: bool,
    pub elapsed: Duration,
}

impl DynamicSceneAssetReloadApplyReport {
    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }

    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }

    pub fn stale_count(&self) -> usize {
        self.stale.len()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct DynamicSceneAssetReloadReadyReport {
    pub(crate) ready: Vec<DynamicSceneAssetReloadResult>,
    pub(crate) stale: Vec<DynamicSceneAssetReloadStaleResult>,
    pub(crate) pending_count: usize,
    pub(crate) collected_bytes: usize,
    pub(crate) inspected_count: usize,
    pub(crate) ready_budget_exhausted: bool,
    pub(crate) ready_budget_overrun: bool,
    pub(crate) elapsed: Duration,
}

impl DynamicSceneAssetReloadReadyReport {
    #[cfg(test)]
    pub(crate) fn ready_count(&self) -> usize {
        self.ready.len()
    }

    #[cfg(test)]
    pub(crate) fn stale_count(&self) -> usize {
        self.stale.len()
    }

    #[cfg(test)]
    pub(crate) fn into_ready(self) -> Vec<DynamicSceneAssetReloadResult> {
        self.ready
    }

    #[cfg(test)]
    pub(crate) fn spawn_ready_into(self, world: &mut World) -> DynamicSceneAssetReloadApplyReport {
        self.spawn_ready_into_budgeted(world, usize::MAX, usize::MAX, Duration::MAX)
            .0
    }

    #[cfg(test)]
    pub(crate) fn spawn_ready_into_level(
        self,
        level: &LevelSystem,
    ) -> DynamicSceneAssetReloadApplyReport {
        self.spawn_ready_into_level_budgeted(level, usize::MAX, usize::MAX, Duration::MAX)
            .0
    }

    #[cfg(test)]
    pub(crate) fn spawn_ready_into_budgeted(
        self,
        world: &mut World,
        max_count: usize,
        max_bytes: usize,
        time_budget: Duration,
    ) -> (
        DynamicSceneAssetReloadApplyReport,
        Vec<DynamicSceneAssetReloadResult>,
    ) {
        let started = Instant::now();
        let mut report = DynamicSceneAssetReloadApplyReport {
            stale: self.stale,
            pending_count: self.pending_count,
            ..DynamicSceneAssetReloadApplyReport::default()
        };
        let mut deferred = Vec::new();

        for result in self.ready {
            let bytes = result.estimated_bytes();
            let attempted = report.applied.len().saturating_add(report.failed.len());
            let count_exhausted = attempted >= max_count;
            let bytes_exhausted = report.applied_bytes.saturating_add(bytes) > max_bytes;
            let time_exhausted = attempted > 0 && started.elapsed() >= time_budget;
            if count_exhausted || bytes_exhausted || time_exhausted {
                deferred.push(result);
                continue;
            }

            report.applied_bytes = report.applied_bytes.saturating_add(bytes);
            match result.spawn_into_with_target_limit(world, max_bytes) {
                Ok(applied) => report.applied.push(applied),
                Err(failure) => report.failed.push(failure),
            }
        }

        finish_apply_report(&mut report, &deferred, started, time_budget);
        (report, deferred)
    }

    #[cfg(test)]
    pub(crate) fn spawn_ready_into_level_budgeted(
        self,
        level: &LevelSystem,
        max_count: usize,
        max_bytes: usize,
        time_budget: Duration,
    ) -> (
        DynamicSceneAssetReloadApplyReport,
        Vec<DynamicSceneAssetReloadResult>,
    ) {
        let started = Instant::now();
        let mut report = DynamicSceneAssetReloadApplyReport {
            stale: self.stale,
            pending_count: self.pending_count,
            ..DynamicSceneAssetReloadApplyReport::default()
        };
        let mut deferred = Vec::new();

        for result in self.ready {
            let bytes = result.estimated_bytes();
            let attempted = report.applied.len().saturating_add(report.failed.len());
            let count_exhausted = attempted >= max_count;
            let bytes_exhausted = report.applied_bytes.saturating_add(bytes) > max_bytes;
            let time_exhausted = attempted > 0 && started.elapsed() >= time_budget;
            if count_exhausted || bytes_exhausted || time_exhausted {
                deferred.push(result);
                continue;
            }

            report.applied_bytes = report.applied_bytes.saturating_add(bytes);
            match result.spawn_into_level_with_target_limit(level, max_bytes) {
                Ok(applied) => report.applied.push(applied),
                Err(failure) => report.failed.push(failure),
            }
        }

        finish_apply_report(&mut report, &deferred, started, time_budget);
        (report, deferred)
    }
}

#[cfg(test)]
fn finish_apply_report(
    report: &mut DynamicSceneAssetReloadApplyReport,
    deferred: &[DynamicSceneAssetReloadResult],
    started: Instant,
    time_budget: Duration,
) {
    report.deferred_count = deferred.len();
    report.apply_budget_exhausted = !deferred.is_empty();
    report.elapsed = started.elapsed();
    report.apply_budget_overrun = report.elapsed > time_budget;
}

#[derive(Clone, Debug, Default, PartialEq)]
/// 串联事件接纳和目标应用的帧报告，供调用方展示同一 tick 的处理边界。
pub struct DynamicSceneAssetReloadFrameApplyReport {
    pub drain: DynamicSceneAssetReloadDrainReport,
    pub apply: DynamicSceneAssetReloadApplyReport,
}

impl DynamicSceneAssetReloadFrameApplyReport {
    pub fn events_drained(&self) -> usize {
        self.drain.events_drained
    }

    pub fn scheduled_count(&self) -> usize {
        self.drain.scheduled
    }

    pub fn skipped_count(&self) -> usize {
        self.drain.skipped_count()
    }

    pub fn skipped_count_for(&self, reason: DynamicSceneAssetReloadSkipReason) -> usize {
        self.drain.skipped_count_for(reason)
    }

    pub fn superseded_pending_count(&self) -> usize {
        self.drain.superseded_pending_count()
    }

    pub fn applied_count(&self) -> usize {
        self.apply.applied_count()
    }

    pub fn failed_count(&self) -> usize {
        self.apply.failed_count()
    }

    pub fn stale_count(&self) -> usize {
        self.apply.stale_count()
    }

    pub fn receiver_disconnected(&self) -> bool {
        self.drain.receiver_disconnected
    }

    pub fn pending_count(&self) -> usize {
        self.apply.pending_count
    }
}

#[cfg(test)]
#[path = "tests/reports.rs"]
mod tests;
