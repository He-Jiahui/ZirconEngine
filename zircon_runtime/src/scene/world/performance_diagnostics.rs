use crate::scene::ecs::{
    ChangeDetectionScanStats, EcsFramePerformanceDiagnostics, QueryStateCacheStats,
};

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
}
