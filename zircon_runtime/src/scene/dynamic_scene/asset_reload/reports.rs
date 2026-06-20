use crate::{
    core::framework::tasks::AsyncTaskState,
    scene::{LevelSystem, World},
};

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
pub struct DynamicSceneAssetReloadDrainReport {
    pub events_drained: usize,
    pub scheduled: usize,
    pub skipped: Vec<DynamicSceneAssetReloadSkippedEvent>,
    pub superseded_pending: Vec<DynamicSceneAssetReloadSupersededTask>,
    pub receiver_disconnected: bool,
    pub pending_count: usize,
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

    pub fn count_for_state(&self, state: AsyncTaskState) -> usize {
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
pub struct DynamicSceneAssetReloadApplyReport {
    pub applied: Vec<DynamicSceneAssetReloadAppliedScene>,
    pub failed: Vec<DynamicSceneAssetReloadApplyFailure>,
    pub stale: Vec<DynamicSceneAssetReloadStaleResult>,
    pub pending_count: usize,
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
pub struct DynamicSceneAssetReloadReadyReport {
    pub ready: Vec<DynamicSceneAssetReloadResult>,
    pub stale: Vec<DynamicSceneAssetReloadStaleResult>,
    pub pending_count: usize,
}

impl DynamicSceneAssetReloadReadyReport {
    pub fn ready_count(&self) -> usize {
        self.ready.len()
    }

    pub fn stale_count(&self) -> usize {
        self.stale.len()
    }

    pub fn into_ready(self) -> Vec<DynamicSceneAssetReloadResult> {
        self.ready
    }

    pub fn spawn_ready_into(self, world: &mut World) -> DynamicSceneAssetReloadApplyReport {
        let mut report = DynamicSceneAssetReloadApplyReport {
            stale: self.stale,
            pending_count: self.pending_count,
            ..DynamicSceneAssetReloadApplyReport::default()
        };

        for result in self.ready {
            match result.spawn_into(world) {
                Ok(applied) => report.applied.push(applied),
                Err(failure) => report.failed.push(failure),
            }
        }

        report
    }

    pub fn spawn_ready_into_level(self, level: &LevelSystem) -> DynamicSceneAssetReloadApplyReport {
        level.with_world_mut(|world| self.spawn_ready_into(world))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DynamicSceneAssetReloadTickReport {
    pub drain: DynamicSceneAssetReloadDrainReport,
    pub ready: DynamicSceneAssetReloadReadyReport,
}

impl DynamicSceneAssetReloadTickReport {
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

    pub fn ready_count(&self) -> usize {
        self.ready.ready_count()
    }

    pub fn stale_count(&self) -> usize {
        self.ready.stale_count()
    }

    pub fn receiver_disconnected(&self) -> bool {
        self.drain.receiver_disconnected
    }

    pub fn pending_count(&self) -> usize {
        self.ready.pending_count
    }

    pub fn into_ready(self) -> Vec<DynamicSceneAssetReloadResult> {
        self.ready.into_ready()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
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
