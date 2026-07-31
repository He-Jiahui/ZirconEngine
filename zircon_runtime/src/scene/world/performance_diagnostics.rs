use crate::scene::ecs::{
    ChangeDetectionScanStats, EcsFramePerformanceDiagnostics, NativeSystemCallbackTiming,
    QueryStateCacheStats,
};
use std::time::Duration;

use super::World;

impl World {
    pub fn reset_ecs_frame_performance_diagnostics(&mut self) {
        self.ecs_frame_performance_diagnostics = EcsFramePerformanceDiagnostics::default();
    }

    pub fn ecs_frame_performance_diagnostics(&self) -> EcsFramePerformanceDiagnostics {
        self.ecs_frame_performance_diagnostics
    }

    pub(crate) fn record_ecs_query_cache_stats(&mut self, stats: QueryStateCacheStats) {
        self.ecs_frame_performance_diagnostics
            .add_query_stats(stats);
    }

    pub(crate) fn record_ecs_change_detection_stats(&mut self, stats: ChangeDetectionScanStats) {
        self.ecs_frame_performance_diagnostics
            .add_change_detection_stats(stats);
    }

    pub(crate) fn record_native_system_conflicts(&mut self, count: usize) {
        self.ecs_frame_performance_diagnostics
            .native_system_schedule_mut()
            .record_conflicts(count);
    }

    pub(crate) fn record_native_system_main_callback(
        &mut self,
        callback: Duration,
        conservative_world_writer: bool,
    ) {
        self.ecs_frame_performance_diagnostics
            .native_system_schedule_mut()
            .record_main_callback(callback, conservative_world_writer);
    }

    pub(crate) fn record_native_system_worker_batch(
        &mut self,
        timings: &[NativeSystemCallbackTiming],
        elapsed: Duration,
        scheduler_parallelism: usize,
    ) {
        self.ecs_frame_performance_diagnostics
            .native_system_schedule_mut()
            .record_worker_batch(timings, elapsed, scheduler_parallelism);
    }
}
