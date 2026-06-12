use super::QueryState;

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
