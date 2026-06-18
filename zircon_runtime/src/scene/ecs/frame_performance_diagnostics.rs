use crate::core::diagnostics::DiagnosticStore;

use super::{ChangeDetectionScanStats, QueryStateCacheStats};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EcsFramePerformanceDiagnostics {
    pub query: QueryStateCacheStats,
    pub change_detection: ChangeDetectionScanStats,
}

impl EcsFramePerformanceDiagnostics {
    pub fn new(query: QueryStateCacheStats, change_detection: ChangeDetectionScanStats) -> Self {
        Self {
            query,
            change_detection,
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

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        self.query.record_diagnostics(store, frame_index);
        self.change_detection.record_diagnostics(store, frame_index);
    }
}
