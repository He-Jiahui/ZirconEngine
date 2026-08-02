use std::time::Duration;

/// Hard memory and per-frame work limits for dynamic scene hot reload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicSceneAssetReloadLimits {
    pub max_events_per_tick: usize,
    pub max_event_bytes_per_tick: usize,
    pub max_schedules_per_tick: usize,
    pub max_ready_per_tick: usize,
    pub max_apply_per_tick: usize,
    pub max_active_tasks: usize,
    pub max_latest_entries: usize,
    pub max_pending_metadata_bytes: usize,
    pub max_prepared_scene_bytes: usize,
    pub max_pending_result_bytes: usize,
    pub max_ready_bytes_per_tick: usize,
    pub max_apply_bytes_per_tick: usize,
    pub event_time_budget: Duration,
    pub ready_time_budget: Duration,
    pub apply_time_budget: Duration,
    pub latest_revision_ttl: Duration,
}

impl DynamicSceneAssetReloadLimits {
    pub(crate) fn normalized(mut self) -> Self {
        const MIN_BOUNDED_RESULT_BYTES: usize = 1_024;

        self.max_active_tasks = self.max_active_tasks.max(1);
        self.max_event_bytes_per_tick = self.max_event_bytes_per_tick.max(1);
        self.max_latest_entries = self.max_latest_entries.max(self.max_active_tasks);
        self.max_pending_metadata_bytes = self.max_pending_metadata_bytes.max(1);
        self.max_pending_result_bytes = self.max_pending_result_bytes.max(MIN_BOUNDED_RESULT_BYTES);
        self.max_ready_bytes_per_tick = self.max_ready_bytes_per_tick.max(MIN_BOUNDED_RESULT_BYTES);
        self.max_apply_bytes_per_tick = self.max_apply_bytes_per_tick.max(MIN_BOUNDED_RESULT_BYTES);

        let per_active_result_limit = self
            .max_pending_result_bytes
            .checked_div(self.max_active_tasks)
            .unwrap_or(1)
            .max(1);
        self.max_prepared_scene_bytes = self
            .max_prepared_scene_bytes
            .max(1)
            .min(per_active_result_limit)
            .min(self.max_ready_bytes_per_tick)
            .min(self.max_apply_bytes_per_tick);
        self
    }
}

impl Default for DynamicSceneAssetReloadLimits {
    fn default() -> Self {
        Self {
            max_events_per_tick: 64,
            max_event_bytes_per_tick: 512 * 1024,
            max_schedules_per_tick: 16,
            max_ready_per_tick: 16,
            max_apply_per_tick: 4,
            max_active_tasks: 32,
            max_latest_entries: 4_096,
            max_pending_metadata_bytes: 4 * 1024 * 1024,
            max_prepared_scene_bytes: 16 * 1024 * 1024,
            max_pending_result_bytes: 256 * 1024 * 1024,
            max_ready_bytes_per_tick: 32 * 1024 * 1024,
            max_apply_bytes_per_tick: 32 * 1024 * 1024,
            event_time_budget: Duration::from_millis(2),
            ready_time_budget: Duration::from_millis(2),
            apply_time_budget: Duration::from_millis(2),
            latest_revision_ttl: Duration::from_secs(60),
        }
    }
}
