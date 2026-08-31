use crate::scene::components::Name;
use crate::scene::world::World;
use crate::scene::EntityId;

use super::index::{CompiledDescendantNameEntry, CompiledDescendantNameIndex};

impl World {
    /// Returns the generation used to invalidate compiled scene topology bindings.
    pub fn scene_binding_generation(&self, root: EntityId) -> u64 {
        self.scene_binding_generations.for_root(root)
    }

    /// Returns the topology-wide generation used to retry bindings whose
    /// target did not exist at their compile boundary.
    pub(crate) fn scene_binding_catalog_generation(&self) -> u64 {
        self.scene_binding_generations.catalog_generation()
    }

    /// Compiles a dense, hierarchy-ordered name projection for a root's descendants.
    ///
    /// The projection belongs to the scene runtime. Consumers may retain it across
    /// frames and must recompile only when the root's
    /// [`Self::scene_binding_generation`] changes.
    pub fn compile_descendant_name_index(
        &self,
        root: EntityId,
    ) -> Option<CompiledDescendantNameIndex> {
        if !self.contains_entity(root) {
            return None;
        }

        let entries = self
            .subtree_entity_ids(root)
            .into_iter()
            .filter(|entity| *entity != root)
            .filter_map(|entity| {
                self.get::<Name>(entity).map(|name| {
                    CompiledDescendantNameEntry::new(entity, name.0.clone().into_boxed_str())
                })
            })
            .collect();

        Some(CompiledDescendantNameIndex::new(
            root,
            self.scene_binding_generation(root),
            entries,
        ))
    }

    pub(in crate::scene::world) fn advance_scene_binding_generation_for_name(
        &mut self,
        entity: EntityId,
    ) {
        self.advance_scene_binding_generations(Some(entity));
    }

    pub(in crate::scene::world) fn advance_scene_binding_generations_for_reparent(
        &mut self,
        entity: EntityId,
        previous_parent: Option<EntityId>,
        current_parent: Option<EntityId>,
    ) {
        let mut roots = self.scene_binding_ancestor_chain(Some(entity));
        roots.extend(self.scene_binding_ancestor_chain(previous_parent));
        roots.extend(self.scene_binding_ancestor_chain(current_parent));
        roots.sort_unstable();
        roots.dedup();
        self.scene_binding_generations.advance_roots(roots);
    }

    pub(in crate::scene::world) fn advance_scene_binding_generations_for_removal(
        &mut self,
        entity: EntityId,
        previous_parent: Option<EntityId>,
    ) {
        // The removed entity no longer has a hierarchy edge, but its identifier
        // can be inserted again. Advance that tombstone root as well as the old
        // ancestor chain so retained bindings cannot cross entity lifetimes.
        // A valid hierarchy is acyclic, so the tombstone and its old ancestor chain are unique.
        let ancestors = self.scene_binding_ancestor_chain(previous_parent);
        self.scene_binding_generations
            .advance_roots(scene_binding_removal_roots(entity, ancestors));
    }

    pub(in crate::scene::world) fn advance_scene_binding_generations_for_new_descendant(
        &mut self,
        entity: EntityId,
    ) {
        self.advance_scene_binding_generations(Some(entity));
    }

    pub(in crate::scene::world) fn invalidate_all_scene_binding_generations(&mut self) {
        let entities = self.stable_entity_ids().collect::<Vec<_>>();
        self.scene_binding_generations.advance_roots(entities);
    }

    /// Makes every topology binding in this replacement world stale relative
    /// to the retired world, even when entity identifiers are reused.
    pub(in crate::scene) fn advance_scene_binding_generations_after(&mut self, previous: &World) {
        let entities = self.stable_entity_ids().collect::<Vec<_>>();
        self.scene_binding_generations
            .advance_roots_after(&previous.scene_binding_generations, entities);
    }

    fn advance_scene_binding_generations(&mut self, first_root: Option<EntityId>) {
        self.scene_binding_generations
            .advance_roots(self.scene_binding_ancestor_chain(first_root));
    }

    fn scene_binding_ancestor_chain(&self, first_root: Option<EntityId>) -> Vec<EntityId> {
        let mut roots = Vec::new();
        let mut current = first_root;
        let mut remaining = self.entities.len().saturating_add(1);
        while let Some(entity) = current {
            if remaining == 0 {
                break;
            }
            roots.push(entity);
            current = self.parent_of(entity);
            remaining -= 1;
        }
        roots
    }
}

fn scene_binding_removal_roots(
    entity: EntityId,
    ancestors: impl IntoIterator<Item = EntityId>,
) -> impl Iterator<Item = EntityId> {
    std::iter::once(entity).chain(ancestors)
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::scene::{EntityId, NodeKind, World};

    use super::super::SceneBindingGenerations;
    use super::scene_binding_removal_roots;

    const BENCHMARK_ROOT_COUNT: usize = 65_536;
    const BENCHMARK_WARMUP_PAIRS: usize = 4;
    const BENCHMARK_SAMPLE_PAIRS: usize = 21;

    #[test]
    fn streamed_scene_binding_root_invalidation_preserves_one_generation() {
        let mut world = World::empty();
        let root = world.spawn_node(NodeKind::Empty).unwrap();
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        let removed = world.spawn_node(NodeKind::Empty).unwrap();
        world.set_parent_checked(parent, Some(root)).unwrap();
        world.set_parent_checked(removed, Some(parent)).unwrap();
        let previous_generation = world.scene_binding_generation(root);

        world.remove_entity(removed).unwrap();

        let generation = world.scene_binding_generation(root);
        assert!(generation > previous_generation);
        assert_eq!(world.scene_binding_generation(parent), generation);
        assert_eq!(world.scene_binding_generation(removed), generation);
    }

    #[test]
    #[ignore = "performance acceptance benchmark"]
    fn streamed_scene_binding_root_invalidation_performance_acceptance() {
        let roots = benchmark_roots(BENCHMARK_ROOT_COUNT);
        let entity = roots[0];
        let ancestors = &roots[1..];
        let mut legacy = SceneBindingGenerations::default();
        let mut optimized = SceneBindingGenerations::default();
        legacy.advance_roots(roots.iter().copied());
        optimized.advance_roots(roots.iter().copied());

        for _ in 0..BENCHMARK_WARMUP_PAIRS {
            black_box(time_legacy(&mut legacy, entity, ancestors));
            black_box(time_optimized(&mut optimized, entity, ancestors));
        }

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut legacy_checksum = 0_u64;
        let mut optimized_checksum = 0_u64;
        for pair in 0..BENCHMARK_SAMPLE_PAIRS {
            let ((legacy_ns, legacy_result), (optimized_ns, optimized_result)) = if pair % 2 == 0 {
                (
                    time_legacy(&mut legacy, entity, ancestors),
                    time_optimized(&mut optimized, entity, ancestors),
                )
            } else {
                let optimized_result = time_optimized(&mut optimized, entity, ancestors);
                let legacy_result = time_legacy(&mut legacy, entity, ancestors);
                (legacy_result, optimized_result)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
            legacy_checksum = legacy_checksum.wrapping_add(legacy_result);
            optimized_checksum = optimized_checksum.wrapping_add(optimized_result);
        }

        let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
        let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
        let optimized_p50_ns = nearest_rank(&optimized_samples, 50);
        let optimized_p95_ns = nearest_rank(&optimized_samples, 95);

        println!(
            "RUNTIME05_STREAMED_SCENE_BINDING_ROOT_INVALIDATION_PERF roots={} warmup_pairs={} sample_pairs={} order=alternating percentile=nearest-rank legacy_sort_calls=1 legacy_dedup_calls=1 optimized_sort_calls=0 optimized_dedup_calls=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_checksum={} optimized_checksum={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
            BENCHMARK_ROOT_COUNT,
            BENCHMARK_WARMUP_PAIRS,
            BENCHMARK_SAMPLE_PAIRS,
            legacy_p50_ns,
            legacy_p95_ns,
            optimized_p50_ns,
            optimized_p95_ns,
            legacy_checksum,
            optimized_checksum,
            legacy_samples,
            optimized_samples,
        );

        assert_eq!(legacy_checksum, optimized_checksum);
        assert_ne!(optimized_checksum, 0);
        assert!(
            optimized_p50_ns.saturating_mul(100) <= legacy_p50_ns.saturating_mul(90),
            "streamed roots must reduce P50 by at least 10%: legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns",
        );
        assert!(
            optimized_p95_ns <= legacy_p95_ns,
            "streamed roots must not regress P95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns",
        );
    }

    fn benchmark_roots(count: usize) -> Vec<EntityId> {
        (0..count)
            .map(|index| ((index * 32_771) % count) as EntityId + 1)
            .collect()
    }

    fn time_legacy(
        generations: &mut SceneBindingGenerations,
        entity: EntityId,
        ancestors: &[EntityId],
    ) -> (u128, u64) {
        let started = Instant::now();
        let mut roots = Vec::with_capacity(ancestors.len() + 1);
        roots.push(entity);
        roots.extend_from_slice(black_box(ancestors));
        roots.sort_unstable();
        roots.dedup();
        generations.advance_roots(roots);
        let elapsed = started.elapsed().as_nanos();
        (
            elapsed,
            generations
                .for_root(entity)
                .wrapping_add(generations.catalog_generation()),
        )
    }

    fn time_optimized(
        generations: &mut SceneBindingGenerations,
        entity: EntityId,
        ancestors: &[EntityId],
    ) -> (u128, u64) {
        let started = Instant::now();
        generations.advance_roots(scene_binding_removal_roots(
            entity,
            black_box(ancestors).iter().copied(),
        ));
        let elapsed = started.elapsed().as_nanos();
        (
            elapsed,
            generations
                .for_root(entity)
                .wrapping_add(generations.catalog_generation()),
        )
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100).max(1);
        sorted[rank - 1]
    }
}
