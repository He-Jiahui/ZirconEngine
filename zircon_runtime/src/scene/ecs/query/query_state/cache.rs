use crate::scene::ecs::{
    ArchetypeIndexPerformanceStats, QueryDataAccess, QueryFilter, StorageType,
};
use crate::scene::World;

use super::{CachedArchetypePlan, QueryComponentBinding, QueryState};

impl<D, F> QueryState<D, F>
where
    D: QueryDataAccess,
    F: QueryFilter,
{
    pub fn update_cache(&mut self, world: &World) {
        let archetype_generation = world.archetype_generation();
        if self.cached_archetype_generation == archetype_generation {
            self.refresh_plan_memberships(world);
            self.cache_hits = self.cache_hits.saturating_add(1);
            return;
        }

        if self.cached_archetype_generation != u64::MAX
            && self.cached_archetype_generation < archetype_generation
        {
            let index_stats_before = world.query_archetype_index_performance_stats();
            let new_matches = world.matching_query_archetypes_from(
                &self.access,
                self.cached_archetype_generation as usize,
            );
            self.record_archetype_index_work(
                index_stats_before,
                world.query_archetype_index_performance_stats(),
            );
            self.cached_archetype_generation = archetype_generation;
            if new_matches.is_empty() {
                self.refresh_plan_memberships(world);
                self.cache_hits = self.cache_hits.saturating_add(1);
                return;
            }

            let compiled_new_plans = new_matches
                .into_iter()
                .map(|archetype| self.compile_archetype_plan(world, archetype))
                .collect::<Vec<_>>();
            self.cached_archetype_plans.extend(compiled_new_plans);
            self.refresh_plan_memberships(world);
            self.cache_misses = self.cache_misses.saturating_add(1);
            self.cache_rebuilds = self.cache_rebuilds.saturating_add(1);
            return;
        }

        let index_stats_before = world.query_archetype_index_performance_stats();
        let matched_archetypes = world.matching_query_archetypes(&self.access);
        self.record_archetype_index_work(
            index_stats_before,
            world.query_archetype_index_performance_stats(),
        );
        let compiled_plans = matched_archetypes
            .iter()
            .copied()
            .map(|archetype| self.compile_archetype_plan(world, archetype))
            .collect::<Vec<_>>();
        self.cached_archetype_plans = compiled_plans;
        let candidate_count = world.matching_query_archetype_entity_count(&matched_archetypes);
        self.cached_archetype_generation = archetype_generation;
        self.cached_entity_count = candidate_count;
        self.cache_misses = self.cache_misses.saturating_add(1);
        self.cache_rebuilds = self.cache_rebuilds.saturating_add(1);
        self.last_candidate_entity_count = candidate_count;
        self.last_matched_entity_count = candidate_count;
    }

    pub fn cached_archetype_count(&self) -> usize {
        self.cached_archetype_plans.len()
    }

    pub(crate) fn cached_archetype_plans(&self) -> &[CachedArchetypePlan] {
        &self.cached_archetype_plans
    }

    pub fn cached_archetype_generation(&self) -> u64 {
        self.cached_archetype_generation
    }

    pub fn cached_entity_count(&self) -> usize {
        self.cached_entity_count
    }

    pub fn cached_revision(&self) -> u64 {
        self.cached_archetype_generation
    }

    pub fn cache_rebuilds(&self) -> u64 {
        self.cache_rebuilds
    }

    fn compile_archetype_plan(
        &mut self,
        world: &World,
        archetype: crate::scene::ecs::ArchetypeId,
    ) -> CachedArchetypePlan {
        self.archetype_plan_compilations = self.archetype_plan_compilations.saturating_add(1);
        let mut bindings = Vec::with_capacity(self.access.reads().len());
        for component_id in self.access.reads().iter().copied() {
            self.archetype_component_membership_checks =
                self.archetype_component_membership_checks.saturating_add(1);
            if !world.query_archetype_contains_component(archetype, component_id) {
                continue;
            }
            let rust_type_id = world
                .query_component_rust_type_id(component_id)
                .expect("matching query component must retain a registered Rust type");
            let binding = match world.query_component_storage_type(component_id) {
                Some(StorageType::Table) => {
                    self.table_column_slot_bindings =
                        self.table_column_slot_bindings.saturating_add(1);
                    QueryComponentBinding::Table {
                        component_id,
                        rust_type_id,
                        column_slot: world
                            .query_archetype_column_slot(archetype, component_id)
                            .expect(
                                "matching dense query component must own a compiled column slot",
                            ),
                    }
                }
                Some(StorageType::SparseSet) => {
                    self.sparse_component_bindings =
                        self.sparse_component_bindings.saturating_add(1);
                    QueryComponentBinding::SparseSet {
                        component_id,
                        rust_type_id,
                    }
                }
                None => continue,
            };
            bindings.push(binding);
        }
        CachedArchetypePlan::new(
            archetype,
            world
                .query_archetype_membership_generation(archetype)
                .expect("matching query archetype must retain a membership generation"),
            bindings,
        )
    }

    fn record_archetype_index_work(
        &mut self,
        before: ArchetypeIndexPerformanceStats,
        after: ArchetypeIndexPerformanceStats,
    ) {
        let delta = after.saturating_delta_since(before);
        self.archetype_index_component_probes = self
            .archetype_index_component_probes
            .saturating_add(delta.component_index_probes);
        self.archetype_index_signature_membership_checks = self
            .archetype_index_signature_membership_checks
            .saturating_add(delta.signature_membership_checks);
    }

    fn refresh_plan_memberships(&mut self, world: &World) {
        let mut candidate_count = 0_usize;
        for plan in &mut self.cached_archetype_plans {
            let archetype = plan.archetype_id();
            if let Some(generation) = world.query_archetype_membership_generation(archetype) {
                plan.refresh_membership_generation(generation);
            }
            candidate_count =
                candidate_count.saturating_add(world.query_archetype_entity_count(archetype));
        }
        self.cached_entity_count = candidate_count;
        self.last_candidate_entity_count = candidate_count;
        self.last_matched_entity_count = candidate_count;
    }
}
