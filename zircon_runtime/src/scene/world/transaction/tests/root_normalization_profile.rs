//! Paired prepare timings on real Worlds; the frozen implementation runs only on valid forests.

use super::*;
use std::hint::black_box;
use std::time::Instant;

const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;

// Complete preimage API body: only this test-local method name and visibility differ.
impl World {
    fn frozen_prepare_entity_subtrees(
        &mut self,
        roots: impl IntoIterator<Item = EntityId>,
    ) -> SceneResult<PreparedEntitySubtrees> {
        self.flush_deferred_component_mutations();
        let world_generation = self.world_generation();
        if world_generation == u64::MAX {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(SceneError::DetachedEntityPreparationGenerationExhausted);
        }
        let hierarchy_index_rebuild_rows = self.ensure_hierarchy_mutation_index_current();
        let roots = roots.into_iter().collect::<BTreeSet<_>>();
        if roots.is_empty() {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(SceneError::DetachedEntityBatchInvariant {
                reason: "detached entity root set is empty",
            });
        }
        for root in roots.iter().copied() {
            if !self.contains_entity(root) {
                self.record_detached_entity_batch_rejected_preflight();
                return Err(SceneError::missing_entity("detach", root));
            }
        }

        let mut normalized_roots = roots
            .iter()
            .copied()
            .filter(|root| {
                let mut parent = self.parent_of(*root);
                while let Some(candidate) = parent {
                    if roots.contains(&candidate) {
                        return false;
                    }
                    parent = self.parent_of(candidate);
                }
                true
            })
            .collect::<Vec<_>>();
        normalized_roots.sort_unstable_by_key(|root| {
            self.stable_entity_order(*root)
                .expect("validated detached root must retain stable order")
        });
        let mut entities_by_order = BTreeMap::new();
        let mut detach_preorder = Vec::new();
        for root in normalized_roots.iter().copied() {
            for entity in self.subtree_entity_ids(root) {
                let order = self
                    .stable_entity_order(entity)
                    .expect("indexed detached entity must retain stable order");
                let previous = entities_by_order.insert(order, entity);
                debug_assert!(previous.is_none() || previous == Some(entity));
                if previous.is_none() {
                    detach_preorder.push(entity);
                }
            }
        }
        let entities = entities_by_order.into_values().collect::<Vec<_>>();
        let stable_batch_indices = entities
            .iter()
            .copied()
            .enumerate()
            .map(|(index, entity)| (entity, index))
            .collect::<HashMap<_, _>>();
        let restore_order = detach_preorder
            .iter()
            .map(|entity| {
                *stable_batch_indices
                    .get(entity)
                    .expect("detached preorder entity must exist in stable batch order")
            })
            .collect::<Vec<_>>();
        if let Err(error) = self.preflight_detached_entities(&entities) {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(error);
        }

        let affected_camera_count =
            self.registered_component_id::<CameraComponent>()
                .map_or(0, |component_id| {
                    entities
                        .iter()
                        .filter(|entity| self.contains_component_id(**entity, component_id))
                        .count()
                });
        Ok(PreparedEntitySubtrees {
            owner: self.detach_preparation_owner.clone(),
            world_generation,
            normalized_roots,
            entities,
            detach_preorder,
            restore_order,
            world_camera_count: self.camera_count(),
            affected_camera_count,
            hierarchy_index_rebuild_rows,
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    Separate,
    Shared,
    Covered,
}

fn fixture(
    roots: usize,
    depth: usize,
    unrelated: usize,
    shape: Shape,
) -> (World, Vec<EntityId>, usize) {
    let mut seed = World::empty();
    let seed_id = seed.spawn_node(NodeKind::Empty).unwrap().to_string();
    let mut document = serde_json::to_value(&seed).unwrap();
    let fields = document.as_object_mut().unwrap();
    let affected_domain = match shape {
        Shape::Separate => roots,
        Shape::Shared => depth + roots,
        Shape::Covered => depth,
    };
    let count = affected_domain + unrelated;
    // Real persistent rows avoid quadratic checked/raw mutation work during deep setup.
    for value in fields.values_mut() {
        if let Some(rows) = value.as_object_mut() {
            if let Some(template) = rows.get(&seed_id).cloned() {
                rows.clear();
                for entity in 1..=count {
                    rows.insert(entity.to_string(), template.clone());
                }
            }
        }
    }
    fields.insert(
        "entities".into(),
        serde_json::json!((1..=count as u64).collect::<Vec<_>>()),
    );
    fields.insert("next_id".into(), serde_json::json!(count as u64 + 1));
    let hierarchy = fields
        .get_mut("hierarchy")
        .unwrap()
        .as_object_mut()
        .unwrap();
    for entity in 1..=affected_domain {
        let parent = match shape {
            Shape::Separate => None,
            Shape::Shared if entity > depth => Some(depth as u64),
            _ if entity > 1 => Some(entity as u64 - 1),
            _ => None,
        };
        hierarchy.insert(entity.to_string(), serde_json::json!({"parent": parent}));
    }
    let requested = match shape {
        Shape::Separate => (1..=roots as u64).collect(),
        Shape::Shared => (depth as u64 + 1..=(depth + roots) as u64).collect(),
        Shape::Covered => ((depth - roots + 1) as u64..=depth as u64).collect(),
    };
    let mut world: World = serde_json::from_value(document).unwrap();
    world.ensure_hierarchy_mutation_index_current();
    (world, requested, affected_domain)
}

fn count_parent_reads(world: &World, roots: &[EntityId]) -> (Vec<EntityId>, usize) {
    let requested = roots.iter().copied().collect::<BTreeSet<_>>();
    let mut reads = 0;
    let normalized = super::root_normalization::normalize_roots(&requested, |entity| {
        reads += 1;
        world.parent_of(entity)
    })
    .unwrap();
    (normalized, reads)
}

#[test]
fn detached_parent_cycle_memo_visits_each_reachable_parent_once() {
    for shape in [Shape::Separate, Shape::Shared, Shape::Covered] {
        for roots in [2, 128] {
            let (mut world, requested, unique_parents) = fixture(roots, 2_048, 97, shape);
            let (mut normalized, reads) = count_parent_reads(&world, &requested);
            assert_eq!(reads, unique_parents);
            normalized.sort_unstable_by_key(|entity| world.stable_entity_order(*entity).unwrap());
            let prepared = world
                .prepare_entity_subtrees(requested.iter().copied())
                .unwrap();
            assert_eq!(prepared.normalized_roots(), normalized);
            assert_eq!(prepared.affected_entity_count(), roots);
            drop(prepared);
            let batch = world
                .remove_entity_subtrees(requested.iter().copied())
                .unwrap();
            assert_eq!(batch.entity_ids().collect::<Vec<_>>(), requested);
            world.restore_detached_entity_batch(batch).unwrap();
            assert_eq!(count_parent_reads(&world, &requested).1, unique_parents);
        }
    }
}

fn measure_prepare(
    world: &mut World,
    roots: &[EntityId],
    frozen: bool,
    expected: &PreparedEntitySubtrees,
) -> u128 {
    let started = Instant::now();
    let result = if frozen {
        world.frozen_prepare_entity_subtrees(black_box(roots.iter().copied()))
    } else {
        world.prepare_entity_subtrees(black_box(roots.iter().copied()))
    };
    black_box(&result);
    let elapsed = started.elapsed().as_nanos();
    // Checking and dropping owned preparations are deliberately outside the timed interval.
    let prepared = result.unwrap();
    assert_eq!(prepared.normalized_roots, expected.normalized_roots);
    assert_eq!(prepared.entities, expected.entities);
    assert_eq!(prepared.detach_preorder, expected.detach_preorder);
    assert_eq!(prepared.restore_order, expected.restore_order);
    assert_eq!(
        prepared.affected_camera_count(),
        expected.affected_camera_count()
    );
    assert_eq!(prepared.world_camera_count(), expected.world_camera_count());
    assert_eq!(prepared.world_generation(), expected.world_generation());
    drop(prepared);
    elapsed
}

fn percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let at = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (at(50), at(95), at(99))
}

#[test]
#[ignore = "Release paired detached prepare profile; schedule with the managed Runtime08 scale gates"]
fn runtime62_detached_parent_cycle_release_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run this paired profile in Release"
    );
    for shape in [Shape::Separate, Shape::Shared, Shape::Covered] {
        for roots in [2, 128] {
            let (mut world, requested, unique_parents) = fixture(roots, 4_096, 100_000, shape);
            let (_, parent_reads) = count_parent_reads(&world, &requested);
            assert_eq!(parent_reads, unique_parents);
            let expected = world
                .frozen_prepare_entity_subtrees(requested.iter().copied())
                .unwrap();
            assert_eq!(expected.affected_entity_count(), roots);
            let before = serde_json::to_value(&world).unwrap();
            let generation = world.world_generation();
            let lifecycle = world.lifecycle_visibility_revision();
            world.reset_ecs_frame_performance_diagnostics();
            for pair in 0..WARMUP_PAIRS {
                for frozen in [pair % 2 == 0, pair % 2 != 0] {
                    black_box(measure_prepare(&mut world, &requested, frozen, &expected));
                }
            }
            let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
            let mut current = Vec::with_capacity(SAMPLE_PAIRS);
            for pair in 0..SAMPLE_PAIRS {
                for frozen in [pair % 2 == 0, pair % 2 != 0] {
                    let elapsed = measure_prepare(&mut world, &requested, frozen, &expected);
                    if frozen {
                        baseline.push(elapsed);
                    } else {
                        current.push(elapsed);
                    }
                }
            }
            assert_eq!(serde_json::to_value(&world).unwrap(), before);
            assert_eq!(world.world_generation(), generation);
            assert_eq!(world.lifecycle_visibility_revision(), lifecycle);
            let diagnostics = world
                .ecs_frame_performance_diagnostics()
                .detached_entity_batches;
            assert_eq!(diagnostics.commit_count, 0);
            assert_eq!(diagnostics.rejected_preflights, 0);
            assert_eq!(diagnostics.full_world_clone_bytes, 0);
            assert_eq!(diagnostics.node_record_clone_bytes, 0);
            assert_eq!(diagnostics.rollback_bytes, 0);
            eprintln!("detached_parent_cycle shape={shape:?} roots={roots} depth=4096 unrelated=100000 parent_reads={parent_reads} baseline_p50_p95_p99_ns={:?} current_p50_p95_p99_ns={:?} baseline_raw_ns={baseline:?} current_raw_ns={current:?}", percentiles(&baseline), percentiles(&current));
        }
    }
}
