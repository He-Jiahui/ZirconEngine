use crate::{
    asset::{AssetEvent, AssetEventKind, AssetId, SceneAsset},
    core::JobScheduler,
    scene::dynamic_scene::DynamicSceneSpawnTask,
};

use super::super::{
    super::{reports::DynamicSceneAssetReloadDrainReport, task::DynamicSceneAssetReloadTask},
    locator_metadata_bytes, DeferredReload, DynamicSceneAssetReloadQueue,
    ORDER_ENTRY_METADATA_BYTES,
};

impl DynamicSceneAssetReloadQueue {
    pub(super) fn schedule_reload(
        &mut self,
        scheduler: &JobScheduler,
        asset_id: AssetId,
        deferred: DeferredReload,
        report: &mut DynamicSceneAssetReloadDrainReport,
    ) {
        let task = match self.task_graph_scope.as_ref() {
            Some(scope) => DynamicSceneSpawnTask::schedule_scene_asset_uri_with_limit_in_scope(
                scheduler,
                scope,
                self.project.clone(),
                deferred.uri,
                deferred.label.clone(),
                self.limits.max_prepared_scene_bytes,
            ),
            None => DynamicSceneSpawnTask::schedule_scene_asset_uri_with_limit(
                scheduler,
                self.project.clone(),
                deferred.uri,
                deferred.label.clone(),
                self.limits.max_prepared_scene_bytes,
            ),
        };
        let task = match task {
            Ok(task) => task,
            Err(reason) => {
                report.failed.push(
                    super::super::super::result::DynamicSceneAssetReloadApplyFailure::new(
                        deferred.event,
                        crate::scene::DynamicSceneError::SpawnTaskAdmission {
                            label: deferred.label,
                            reason,
                        },
                    ),
                );
                return;
            }
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

    pub(super) fn defer_reload(
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

    pub(super) fn schedule_deferred(
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
}

pub(super) fn estimate_task_metadata_bytes(event: &AssetEvent<SceneAsset>, label: &str) -> usize {
    std::mem::size_of::<DynamicSceneAssetReloadTask>()
        .saturating_add(label.len())
        .saturating_add(event.locator().map_or(0, locator_metadata_bytes))
        .saturating_add(event.previous_locator().map_or(0, locator_metadata_bytes))
}

pub(super) fn scene_asset_reload_label(
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
