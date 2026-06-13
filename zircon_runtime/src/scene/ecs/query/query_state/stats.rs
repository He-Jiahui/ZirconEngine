use crate::core::diagnostics::DiagnosticStore;

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
    pub fn record_diagnostics(&self, store: &mut DiagnosticStore, frame_index: u64) {
        record_count(
            store,
            ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
            frame_index,
            self.cache_hits,
        );
        record_count(
            store,
            ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC,
            frame_index,
            self.cache_misses,
        );
        record_count(
            store,
            ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
            frame_index,
            self.cache_rebuilds,
        );
        record_count(
            store,
            ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC,
            frame_index,
            self.candidate_entity_count as u64,
        );
        record_count(
            store,
            ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
            frame_index,
            self.matched_entity_count as u64,
        );
    }
}

fn record_count(store: &mut DiagnosticStore, path: &'static str, frame_index: u64, value: u64) {
    store.record(
        path,
        frame_index,
        value as f64,
        Some("count"),
        ["ecs", "query"],
    );
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
}
