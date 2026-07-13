use crate::core::diagnostics::DiagnosticStore;

use crate::scene::ecs::ChangeDetectionScanStats;

use super::QueryState;

pub const ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC: &str = "ecs.query.archetype_cache_hits";
pub const ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC: &str = "ecs.query.archetype_cache_misses";
pub const ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC: &str =
    "ecs.query.archetype_cache_rebuilds";
pub const ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC: &str = "ecs.query.candidate_entities";
pub const ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC: &str = "ecs.query.matched_entities";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueryStateCacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_rebuilds: u64,
    pub cached_revision: u64,
    pub cached_archetype_count: usize,
    pub cached_entity_count: usize,
    pub candidate_entity_count: usize,
    pub matched_entity_count: usize,
}

impl QueryStateCacheStats {
    pub(crate) fn diagnostic_values(&self) -> [(&'static str, f64); 5] {
        [
            (
                ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
                self.cache_hits as f64,
            ),
            (
                ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC,
                self.cache_misses as f64,
            ),
            (
                ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
                self.cache_rebuilds as f64,
            ),
            (
                ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC,
                self.candidate_entity_count as f64,
            ),
            (
                ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
                self.matched_entity_count as f64,
            ),
        ]
    }

    pub fn saturating_delta_since(self, baseline: Self) -> Self {
        let cache_hits = self.cache_hits.saturating_sub(baseline.cache_hits);
        let cache_misses = self.cache_misses.saturating_sub(baseline.cache_misses);
        let cache_rebuilds = self.cache_rebuilds.saturating_sub(baseline.cache_rebuilds);
        let observed_query_activity = cache_hits > 0 || cache_misses > 0 || cache_rebuilds > 0;
        Self {
            cache_hits,
            cache_misses,
            cache_rebuilds,
            cached_revision: self.cached_revision,
            cached_archetype_count: if observed_query_activity {
                self.cached_archetype_count
            } else {
                0
            },
            cached_entity_count: if observed_query_activity {
                self.cached_entity_count
            } else {
                0
            },
            candidate_entity_count: if observed_query_activity {
                self.candidate_entity_count
            } else {
                0
            },
            matched_entity_count: if observed_query_activity {
                self.matched_entity_count
            } else {
                0
            },
        }
    }

    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        for (path, value) in self.diagnostic_values() {
            record_count(store, path, frame_index, value);
        }
    }
}

fn record_count(store: &mut DiagnosticStore, path: &'static str, frame_index: u64, value: f64) {
    store.record(path, frame_index, value, Some("count"), ["ecs", "query"]);
}

impl<D, F> QueryState<D, F> {
    pub fn cache_stats(&self) -> QueryStateCacheStats {
        QueryStateCacheStats {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            cache_rebuilds: self.cache_rebuilds,
            cached_revision: self.cached_revision,
            cached_archetype_count: self.cached_archetypes.len(),
            cached_entity_count: self.cached_entities.len(),
            candidate_entity_count: self.last_candidate_entity_count,
            matched_entity_count: self.last_matched_entity_count,
        }
    }

    pub(crate) fn take_unreported_cache_stats(&mut self) -> QueryStateCacheStats {
        let current = self.cache_stats();
        let delta = current.saturating_delta_since(self.last_reported_cache_stats);
        self.last_reported_cache_stats = current;
        delta
    }

    pub(crate) fn take_unreported_change_detection_stats(&mut self) -> ChangeDetectionScanStats {
        let current = self.change_detection_stats.get();
        let delta = current.saturating_delta_since(self.last_reported_change_detection_stats);
        self.last_reported_change_detection_stats = current;
        delta
    }

    pub(crate) fn record_change_detection_stats(&self, stats: ChangeDetectionScanStats) {
        let mut current = self.change_detection_stats.get();
        current.merge(stats);
        self.change_detection_stats.set(current);
    }
}
