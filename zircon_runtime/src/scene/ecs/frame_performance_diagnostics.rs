use crate::core::diagnostics::{DiagnosticStore, FrameDiagnostics};
use crate::core::CoreHandle;

use super::{ChangeDetectionScanStats, NativeSystemScheduleDiagnostics, QueryStateCacheStats};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EcsFramePerformanceDiagnostics {
    pub query: QueryStateCacheStats,
    pub change_detection: ChangeDetectionScanStats,
    pub native_system_schedule: NativeSystemScheduleDiagnostics,
}

impl EcsFramePerformanceDiagnostics {
    pub fn new(query: QueryStateCacheStats, change_detection: ChangeDetectionScanStats) -> Self {
        Self {
            query,
            change_detection,
            native_system_schedule: NativeSystemScheduleDiagnostics::default(),
        }
    }

    pub fn add_query_stats(&mut self, stats: QueryStateCacheStats) {
        self.query.cache_hits = self.query.cache_hits.saturating_add(stats.cache_hits);
        self.query.cache_misses = self.query.cache_misses.saturating_add(stats.cache_misses);
        self.query.cache_rebuilds = self
            .query
            .cache_rebuilds
            .saturating_add(stats.cache_rebuilds);
        self.query.cached_revision = self.query.cached_revision.max(stats.cached_revision);
        self.query.cached_archetype_count = self
            .query
            .cached_archetype_count
            .saturating_add(stats.cached_archetype_count);
        self.query.cached_entity_count = self
            .query
            .cached_entity_count
            .saturating_add(stats.cached_entity_count);
        self.query.candidate_entity_count = self
            .query
            .candidate_entity_count
            .saturating_add(stats.candidate_entity_count);
        self.query.matched_entity_count = self
            .query
            .matched_entity_count
            .saturating_add(stats.matched_entity_count);
    }

    pub fn add_change_detection_stats(&mut self, stats: ChangeDetectionScanStats) {
        self.change_detection.merge(stats);
    }

    pub(crate) fn native_system_schedule_mut(&mut self) -> &mut NativeSystemScheduleDiagnostics {
        &mut self.native_system_schedule
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        self.query.record_diagnostics(store, frame_index);
        self.change_detection.record_diagnostics(store, frame_index);
        self.native_system_schedule
            .record_diagnostics(store, frame_index);
    }

    pub fn publish(&self, core: &CoreHandle, frame_index: u64) {
        for (path, value) in self.query.diagnostic_values() {
            core.record_diagnostic(path, frame_index, value, Some("count"), ["ecs", "query"]);
        }
        for (path, value) in self.change_detection.diagnostic_values() {
            core.record_diagnostic(
                path,
                frame_index,
                value,
                Some("count"),
                ["ecs", "change_detection"],
            );
        }
        self.native_system_schedule.publish(core, frame_index);
    }
}

impl FrameDiagnostics for EcsFramePerformanceDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "scene.ecs"
    }
}

#[cfg(test)]
mod tests {
    use crate::core::diagnostics::FrameDiagnostics;

    use super::EcsFramePerformanceDiagnostics;

    #[test]
    fn ecs_frame_performance_diagnostics_uses_scene_ecs_frame_domain() {
        let diagnostics = EcsFramePerformanceDiagnostics::default();
        let status = diagnostics.frame_diagnostics_status();

        assert_eq!(status.domain, "scene.ecs");
        assert!(status.available);
        assert_eq!(status.error, None);
    }
}
