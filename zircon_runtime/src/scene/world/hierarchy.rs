use crate::core::math::Transform;

use super::{SceneError, SceneResult, World, transform_validation::validate_transform_for_write};
use crate::scene::EntityId;
use crate::scene::components::{Hierarchy, LocalTransform, Mobility, NodeRecord};
use crate::scene::ecs::LifecycleEventKind;
use zircon_runtime_interface::world_sync::WorldFact;

impl World {
    pub fn remove_entity(&mut self, entity: EntityId) -> SceneResult<()> {
        if !self.entity_dense_rows.contains_key(&entity) {
            return Err(SceneError::missing_entity("remove", entity));
        }
        let _hierarchy_index_rebuild_rows = self.ensure_hierarchy_mutation_index_current();
        let removed_kind = self.kinds.get(&entity).copied();
        let removed_parent = self.parent_of(entity);
        let removed_order = self
            .stable_entity_order(entity)
            .expect("registered entity must retain stable order");
        let orphaned_children = self.direct_child_entity_ids(entity);
        for child in orphaned_children {
            self.insert(child, Hierarchy { parent: None })?;
            self.mark_inspection_subtree_fields_dirty(child);
        }
        self.record_world_fact(WorldFact::Despawned(entity));
        if let Some(internal) = self.internal_entity(entity) {
            let component_ids = self.entity_archetype_component_ids(entity);
            for component_id in &component_ids {
                self.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, *component_id);
                self.trigger_component_lifecycle(
                    LifecycleEventKind::Despawn,
                    entity,
                    *component_id,
                );
                if let Some((type_id, type_name)) =
                    self.component_registry.rust_type_for_id(*component_id)
                {
                    self.removed_component_events
                        .push_type_id(type_id, type_name, entity);
                }
            }
            self.component_storage
                .remove_entity_components(internal, &component_ids);
        }
        self.observers.remove_entity_observers(entity);
        self.remove_entity_from_archetype(entity);
        self.remove_hierarchy_mutation_index_entry(entity, removed_order, removed_parent);
        self.unregister_stable_entity(entity);
        let removed = self.remove_entity_from_dense_storage(entity);
        debug_assert!(removed);
        self.kinds.remove(&entity);
        if let Some(kind) = removed_kind {
            self.record_node_kind_removed(kind);
        }
        if let Some(components) = self.dynamic_components.remove(&entity) {
            for component_id in components.keys() {
                self.advance_dynamic_component_generation(component_id);
            }
        }
        if self.active_camera == entity {
            self.active_camera = self.first_stable_camera_entity().unwrap_or(0);
        }
        self.bump_lifecycle_visibility_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        self.advance_scene_binding_generations_for_removal(entity, removed_parent);
        Ok(())
    }

    pub fn subtree_records(&self, entity: EntityId) -> Vec<NodeRecord> {
        let mut records = Vec::new();
        self.collect_subtree_records(entity, &mut records);
        records
    }

    pub fn set_parent_checked(
        &mut self,
        child: EntityId,
        parent: Option<EntityId>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(child) {
            return Err(SceneError::missing_entity("reparent", child));
        }
        if parent == Some(child) {
            return Err(SceneError::EntityCannotParentItself { entity: child });
        }
        if let Some(parent) = parent {
            if !self.contains_entity(parent) {
                return Err(SceneError::MissingParent { child, parent });
            }
            if self.is_descendant(parent, child) {
                return Err(SceneError::HierarchyCycle { child, parent });
            }
        }
        self.validate_reparent(child, parent)?;
        if self.parent_of(child) == parent {
            return Ok(false);
        }
        self.record_world_fact(WorldFact::Reparented {
            entity: child,
            new_parent: parent,
        });
        self.insert(child, Hierarchy { parent })?;
        self.record_world_fact(WorldFact::Reparented {
            entity: child,
            new_parent: parent,
        });
        Ok(true)
    }

    pub fn update_transform(
        &mut self,
        entity: EntityId,
        transform: Transform,
    ) -> SceneResult<bool> {
        self.ensure_transform_mutable(entity)?;
        let Some(local) = self.get::<LocalTransform>(entity) else {
            return Err(SceneError::MissingRequiredComponent {
                operation: "update transform",
                entity,
                component: "LocalTransform",
            });
        };
        if local.transform == transform {
            return Ok(false);
        }
        validate_transform_for_write(entity, transform)?;
        self.insert(entity, LocalTransform { transform })?;
        Ok(true)
    }

    pub(super) fn validate_mobility_change(
        &self,
        entity: EntityId,
        mobility: Mobility,
    ) -> SceneResult<()> {
        match mobility {
            Mobility::Dynamic => {
                if self.has_direct_child_matching(entity, |child| {
                    if self.mobility(child) == Some(Mobility::Static) {
                        return true;
                    }
                    false
                }) {
                    return Err(SceneError::DynamicMobilityWithStaticChildren { entity });
                }
            }
            Mobility::Static => {
                if let Some(parent) = self.parent_of(entity) {
                    if self.mobility(parent) == Some(Mobility::Dynamic) {
                        return Err(SceneError::StaticMobilityUnderDynamicParent {
                            entity,
                            parent,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_transform_mutable(&self, entity: EntityId) -> SceneResult<()> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update transform for", entity));
        }
        if self.mobility(entity) == Some(Mobility::Static) {
            return Err(SceneError::StaticTransformMutation { entity });
        }
        Ok(())
    }

    fn validate_reparent(&self, child: EntityId, _parent: Option<EntityId>) -> SceneResult<()> {
        if self.mobility(child) == Some(Mobility::Static) {
            return Err(SceneError::StaticReparentMutation { entity: child });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::scene::NodeKind;

    const MOBILITY_BENCH_ENTITY_COUNT: usize = 8_192;
    const MOBILITY_BENCH_DIRECT_CHILD_COUNT: usize = 8;
    const MOBILITY_BENCH_ITERATIONS: usize = 64;
    const MOBILITY_BENCH_SAMPLE_PAIRS: usize = 21;

    fn world_with_direct_children(
        entity_count: usize,
        direct_child_count: usize,
    ) -> (World, EntityId, Vec<EntityId>) {
        assert!(entity_count > direct_child_count);
        let mut world = World::empty();
        let parent = world.spawn_node(NodeKind::Empty);
        let mut direct_children = Vec::with_capacity(direct_child_count);
        for index in 1..entity_count {
            let entity = world.spawn_node(NodeKind::Empty);
            if index <= direct_child_count {
                world.set_parent_checked(entity, Some(parent)).unwrap();
                direct_children.push(entity);
            }
        }
        (world, parent, direct_children)
    }

    fn legacy_has_static_direct_child(world: &World, parent: EntityId) -> bool {
        world.stable_entity_ids().any(|child| {
            world.parent_of(child) == Some(parent)
                && world.mobility(child) == Some(Mobility::Static)
        })
    }

    fn indexed_has_static_direct_child(world: &World, parent: EntityId) -> bool {
        world.has_direct_child_matching(parent, |child| {
            world.mobility(child) == Some(Mobility::Static)
        })
    }

    fn measure_ns(mut workload: impl FnMut()) -> u128 {
        let started = Instant::now();
        for _ in 0..MOBILITY_BENCH_ITERATIONS {
            workload();
        }
        started.elapsed().as_nanos()
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn dynamic_mobility_rejects_static_direct_child_among_unrelated_entities() {
        let (mut world, parent, direct_children) = world_with_direct_children(4_096, 4);
        world.set_mobility(parent, Mobility::Static).unwrap();
        world
            .set_mobility(direct_children[2], Mobility::Static)
            .unwrap();

        assert_eq!(
            world.set_mobility(parent, Mobility::Dynamic),
            Err(SceneError::DynamicMobilityWithStaticChildren { entity: parent })
        );
    }

    #[test]
    #[ignore = "release performance gate; run through the managed Runtime62 batch"]
    fn indexed_mobility_child_validation_release_gate() {
        let (mut world, parent, direct_children) = world_with_direct_children(
            MOBILITY_BENCH_ENTITY_COUNT,
            MOBILITY_BENCH_DIRECT_CHILD_COUNT,
        );
        world.set_mobility(parent, Mobility::Static).unwrap();
        assert_eq!(direct_children.len(), MOBILITY_BENCH_DIRECT_CHILD_COUNT);
        assert!(!legacy_has_static_direct_child(&world, parent));
        assert!(!indexed_has_static_direct_child(&world, parent));

        let mut legacy_samples = Vec::with_capacity(MOBILITY_BENCH_SAMPLE_PAIRS);
        let mut indexed_samples = Vec::with_capacity(MOBILITY_BENCH_SAMPLE_PAIRS);
        for pair in 0..MOBILITY_BENCH_SAMPLE_PAIRS {
            let measure_legacy = || {
                measure_ns(|| {
                    black_box(legacy_has_static_direct_child(
                        black_box(&world),
                        black_box(parent),
                    ));
                })
            };
            let measure_indexed = || {
                measure_ns(|| {
                    black_box(indexed_has_static_direct_child(
                        black_box(&world),
                        black_box(parent),
                    ));
                })
            };
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                indexed_samples.push(measure_indexed());
            } else {
                indexed_samples.push(measure_indexed());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p50_ns = nearest_rank_percentile(&legacy_samples, 50);
        let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
        let indexed_p50_ns = nearest_rank_percentile(&indexed_samples, 50);
        let indexed_p95_ns = nearest_rank_percentile(&indexed_samples, 95);
        let legacy_entity_visits = MOBILITY_BENCH_ENTITY_COUNT * MOBILITY_BENCH_ITERATIONS;
        let indexed_entity_visits = MOBILITY_BENCH_DIRECT_CHILD_COUNT * MOBILITY_BENCH_ITERATIONS;
        let legacy_samples_ns = sample_csv(&legacy_samples);
        let indexed_samples_ns = sample_csv(&indexed_samples);

        println!(
            "PERF-MVP-558 task=runtime62_mobility_child_index sample_pairs={} entity_count={} direct_children={} iterations={} legacy_entity_visits={} indexed_entity_visits={} legacy_p50_ns={} legacy_p95_ns={} indexed_p50_ns={} indexed_p95_ns={} legacy_samples_ns={} indexed_samples_ns={}",
            MOBILITY_BENCH_SAMPLE_PAIRS,
            MOBILITY_BENCH_ENTITY_COUNT,
            MOBILITY_BENCH_DIRECT_CHILD_COUNT,
            MOBILITY_BENCH_ITERATIONS,
            legacy_entity_visits,
            indexed_entity_visits,
            legacy_p50_ns,
            legacy_p95_ns,
            indexed_p50_ns,
            indexed_p95_ns,
            legacy_samples_ns,
            indexed_samples_ns,
        );

        assert_eq!(legacy_entity_visits, 524_288);
        assert_eq!(indexed_entity_visits, 512);
        assert!(
            indexed_p95_ns.saturating_mul(4) <= legacy_p95_ns,
            "indexed P95 {indexed_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
        );
    }
}
