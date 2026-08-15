use crate::core::diagnostics::DiagnosticStore;

use crate::scene::ecs::ChangeDetectionScanStats;

use super::QueryState;

pub const ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC: &str = "ecs.query.archetype_cache_hits";
pub const ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC: &str = "ecs.query.archetype_cache_misses";
pub const ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC: &str =
    "ecs.query.archetype_cache_rebuilds";
pub const ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC: &str = "ecs.query.candidate_entities";
pub const ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC: &str = "ecs.query.matched_entities";
pub const ECS_QUERY_PLAN_COMPILATIONS_DIAGNOSTIC: &str = "ecs.query.plan_compilations";
pub const ECS_QUERY_PLAN_COMPONENT_MEMBERSHIP_CHECKS_DIAGNOSTIC: &str =
    "ecs.query.plan_component_membership_checks";
pub const ECS_QUERY_PLAN_TABLE_BINDINGS_DIAGNOSTIC: &str = "ecs.query.plan_table_bindings";
pub const ECS_QUERY_PLAN_SPARSE_BINDINGS_DIAGNOSTIC: &str = "ecs.query.plan_sparse_bindings";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueryStateCacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_rebuilds: u64,
    pub archetype_plan_compilations: u64,
    pub archetype_component_membership_checks: u64,
    pub table_column_slot_bindings: u64,
    pub sparse_component_bindings: u64,
    pub archetype_index_component_probes: u64,
    pub archetype_index_signature_membership_checks: u64,
    pub cached_revision: u64,
    pub cached_archetype_count: usize,
    pub cached_entity_count: usize,
    pub candidate_entity_count: usize,
    pub matched_entity_count: usize,
}

impl QueryStateCacheStats {
    pub(crate) fn diagnostic_values(&self) -> [(&'static str, f64); 9] {
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
                ECS_QUERY_PLAN_COMPILATIONS_DIAGNOSTIC,
                self.archetype_plan_compilations as f64,
            ),
            (
                ECS_QUERY_PLAN_COMPONENT_MEMBERSHIP_CHECKS_DIAGNOSTIC,
                self.archetype_component_membership_checks as f64,
            ),
            (
                ECS_QUERY_PLAN_TABLE_BINDINGS_DIAGNOSTIC,
                self.table_column_slot_bindings as f64,
            ),
            (
                ECS_QUERY_PLAN_SPARSE_BINDINGS_DIAGNOSTIC,
                self.sparse_component_bindings as f64,
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
        let archetype_plan_compilations = self
            .archetype_plan_compilations
            .saturating_sub(baseline.archetype_plan_compilations);
        let archetype_component_membership_checks = self
            .archetype_component_membership_checks
            .saturating_sub(baseline.archetype_component_membership_checks);
        let table_column_slot_bindings = self
            .table_column_slot_bindings
            .saturating_sub(baseline.table_column_slot_bindings);
        let sparse_component_bindings = self
            .sparse_component_bindings
            .saturating_sub(baseline.sparse_component_bindings);
        let archetype_index_component_probes = self
            .archetype_index_component_probes
            .saturating_sub(baseline.archetype_index_component_probes);
        let archetype_index_signature_membership_checks = self
            .archetype_index_signature_membership_checks
            .saturating_sub(baseline.archetype_index_signature_membership_checks);
        let observed_query_activity = cache_hits > 0
            || cache_misses > 0
            || cache_rebuilds > 0
            || archetype_plan_compilations > 0;
        Self {
            cache_hits,
            cache_misses,
            cache_rebuilds,
            archetype_plan_compilations,
            archetype_component_membership_checks,
            table_column_slot_bindings,
            sparse_component_bindings,
            archetype_index_component_probes,
            archetype_index_signature_membership_checks,
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
    pub(crate) fn estimated_cache_bytes(&self) -> usize {
        let plan_directory_bytes = self
            .cached_archetype_plans
            .capacity()
            .saturating_mul(std::mem::size_of::<super::CachedArchetypePlan>());
        self.cached_archetype_plans
            .iter()
            .fold(plan_directory_bytes, |bytes, plan| {
                bytes.saturating_add(plan.estimated_heap_bytes())
            })
    }

    pub fn cache_stats(&self) -> QueryStateCacheStats {
        QueryStateCacheStats {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            cache_rebuilds: self.cache_rebuilds,
            archetype_plan_compilations: self.archetype_plan_compilations,
            archetype_component_membership_checks: self.archetype_component_membership_checks,
            table_column_slot_bindings: self.table_column_slot_bindings,
            sparse_component_bindings: self.sparse_component_bindings,
            archetype_index_component_probes: self.archetype_index_component_probes,
            archetype_index_signature_membership_checks: self
                .archetype_index_signature_membership_checks,
            cached_revision: self.cached_archetype_generation,
            cached_archetype_count: self.cached_archetype_plans.len(),
            cached_entity_count: self.cached_entity_count,
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
