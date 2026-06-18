use crate::scene::ecs::{
    ChangeDetectionScanStats, EcsFramePerformanceDiagnostics, QueryStateCacheStats,
};

use super::World;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WorldEcsFramePerformanceDiagnostics(EcsFramePerformanceDiagnostics);

impl PartialEq for WorldEcsFramePerformanceDiagnostics {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl World {
    pub fn reset_ecs_frame_performance_diagnostics(&mut self) {
        self.ecs_frame_performance_diagnostics = WorldEcsFramePerformanceDiagnostics::default();
    }

    pub fn ecs_frame_performance_diagnostics(&self) -> EcsFramePerformanceDiagnostics {
        self.ecs_frame_performance_diagnostics.0
    }

    pub(crate) fn record_ecs_query_cache_stats(&mut self, stats: QueryStateCacheStats) {
        self.ecs_frame_performance_diagnostics
            .0
            .add_query_stats(stats);
    }

    pub(crate) fn record_ecs_change_detection_stats(&mut self, stats: ChangeDetectionScanStats) {
        self.ecs_frame_performance_diagnostics
            .0
            .add_change_detection_stats(stats);
    }
}
