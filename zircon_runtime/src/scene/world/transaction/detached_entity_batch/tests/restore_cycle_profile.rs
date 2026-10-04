//! Paired valid-forest preflight and real restore timings for detached union validation.
use super::*;
use std::hint::black_box;
use std::time::Instant;

const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;

// Complete predecessor body; only this test-local method name differs.
impl World {
    fn frozen_preflight_detached_batch(&self, batch: &DetachedEntityBatch) -> SceneResult<()> {
        if batch.entries.is_empty() {
            return Err(SceneError::DetachedEntityBatchInvariant {
                reason: "detached entity batch is empty",
            });
        }
        if batch.restore_order.len() != batch.entries.len() {
            return Err(SceneError::DetachedEntityBatchInvariant {
                reason: "detached entity batch restore plan has the wrong length",
            });
        }
        let mut restore_indices = BTreeSet::new();
        for index in batch.restore_order.iter().copied() {
            if index >= batch.entries.len() || !restore_indices.insert(index) {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch restore plan is not a permutation",
                });
            }
        }
        let mut entity_ids = BTreeSet::new();
        let mut stable_orders = BTreeSet::new();
        for entry in &batch.entries {
            if !entity_ids.insert(entry.entity) || !stable_orders.insert(entry.stable_order) {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch contains duplicate identity metadata",
                });
            }
        }
        self.entity_registry
            .ensure_capacity_for_additional(entity_ids.len())?;
        let hierarchy_component_id = self.registered_component_id::<Hierarchy>();
        for entry in &batch.entries {
            if self.contains_entity(entry.entity)
                || self.entity_registry.contains_stable(entry.entity)
            {
                return Err(SceneError::DuplicateEntity {
                    entity: entry.entity,
                });
            }
            if self.stable_entity_order_is_occupied(entry.stable_order) {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch stable order is already occupied",
                });
            }
            let archetype_id = self
                .archetype_index
                .id_for_signature(&entry.signature)
                .ok_or(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch archetype no longer exists",
                })?;
            self.archetype_index
                .validate_row_components(archetype_id, &entry.table_components)
                .map_err(|_| SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch table row no longer matches its archetype",
                })?;
            for sparse in &entry.sparse_components {
                self.component_storage
                    .validate_transferred_row(sparse.component_id(), sparse)?;
            }
            if let Some(hierarchy_component_id) = hierarchy_component_id {
                let hierarchy = entry
                    .table_components
                    .get(&hierarchy_component_id)
                    .and_then(|(value, _)| value.downcast_ref::<Hierarchy>())
                    .ok_or(SceneError::DetachedEntityBatchInvariant {
                        reason: "detached entity batch is missing its hierarchy boundary",
                    })?;
                if let Some(parent) = hierarchy.parent {
                    if !self.contains_entity(parent) && !entity_ids.contains(&parent) {
                        return Err(SceneError::MissingParent {
                            child: entry.entity,
                            parent,
                        });
                    }
                }
            }
        }
        if let Some(detached_active_camera) = batch.detached_active_camera {
            let active_camera_is_valid = if self.contains_entity(detached_active_camera) {
                self.contains_component::<CameraComponent>(detached_active_camera)
            } else {
                self.registered_component_id::<CameraComponent>()
                    .is_some_and(|camera_component_id| {
                        batch.entries.iter().any(|entry| {
                            entry.entity == detached_active_camera
                                && entry.signature.contains(camera_component_id)
                        })
                    })
            };
            if !active_camera_is_valid {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch active camera is no longer available",
                });
            }
        }
        Ok(())
    }
}

fn fixture(
    roots: usize,
    depth: usize,
    unrelated: usize,
) -> (World, DetachedEntityBatch, Vec<EntityId>) {
    let mut seed = World::empty();
    let seed_id = seed.spawn_node(NodeKind::Empty).unwrap().to_string();
    let mut document = serde_json::to_value(&seed).unwrap();
    let fields = document.as_object_mut().unwrap();
    let affected = depth + roots;
    let count = affected + unrelated;
    // Persisted real World rows avoid quadratic public setup work for a long chain.
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
    for entity in 1..=affected {
        let parent = if entity > depth {
            Some(depth as u64)
        } else if entity > 1 {
            Some(entity as u64 - 1)
        } else {
            None
        };
        hierarchy.insert(entity.to_string(), serde_json::json!({"parent": parent}));
    }
    let requested = (depth as u64 + 1..=affected as u64).collect::<Vec<_>>();
    let mut world: World = serde_json::from_value(document).unwrap();
    world.ensure_hierarchy_mutation_index_current();
    let batch = world
        .remove_entity_subtrees(requested.iter().copied())
        .unwrap();
    assert_eq!(batch.entity_ids().collect::<Vec<_>>(), requested);
    (world, batch, requested)
}

fn batch_parent_overlay(
    world: &World,
    batch: &DetachedEntityBatch,
) -> HashMap<EntityId, Option<EntityId>> {
    let hierarchy_id = world.registered_component_id::<Hierarchy>().unwrap();
    batch
        .entries
        .iter()
        .map(|entry| {
            let hierarchy = entry
                .table_components
                .get(&hierarchy_id)
                .unwrap()
                .0
                .downcast_ref::<Hierarchy>()
                .unwrap();
            (entry.entity, hierarchy.parent)
        })
        .collect()
}

#[test]
fn restore_union_parent_memo_reads_each_reachable_live_ancestor_once() {
    for roots in [2, 128] {
        let depth = 2_048;
        let (world, batch, requested) = fixture(roots, depth, 97);
        let overlay = batch_parent_overlay(&world, &batch);
        let mut live_parent_reads = 0;
        super::restore_parent_validation::validate_restore_parent_chains(
            &overlay,
            requested.iter().copied(),
            |entity| {
                live_parent_reads += 1;
                world.parent_of(entity)
            },
        )
        .unwrap();
        assert_eq!(overlay.len(), roots);
        assert_eq!(live_parent_reads, depth);
        assert_eq!(world.stable_entity_ids().count(), depth + 97);
    }
}

fn measure_preflight(world: &World, batch: &DetachedEntityBatch, frozen: bool) -> u128 {
    let started = Instant::now();
    let result = if frozen {
        world.frozen_preflight_detached_batch(black_box(batch))
    } else {
        world.preflight_detached_batch(black_box(batch))
    };
    black_box(&result);
    let elapsed = started.elapsed().as_nanos();
    assert!(
        result.is_ok(),
        "valid detached forest must pass both complete preflights"
    );
    elapsed
}

fn percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let at = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (at(50), at(95), at(99))
}

#[test]
#[ignore = "Release paired restore preflight and real restore profile; schedule in managed Runtime08/62 lane"]
fn runtime62_detached_restore_union_cycle_release_profile() {
    assert!(
        !cfg!(debug_assertions),
        "run the detached restore profile in Release"
    );
    for roots in [2, 128] {
        let depth = 4_096;
        let (mut world, mut batch, requested) = fixture(roots, depth, 100_000);
        let overlay = batch_parent_overlay(&world, &batch);
        let mut live_parent_reads = 0;
        super::restore_parent_validation::validate_restore_parent_chains(
            &overlay,
            requested.iter().copied(),
            |entity| {
                live_parent_reads += 1;
                world.parent_of(entity)
            },
        )
        .unwrap();
        assert_eq!(live_parent_reads, depth);
        let stored = serde_json::to_value(&world).unwrap();
        let generation = world.world_generation();
        world.reset_ecs_frame_performance_diagnostics();
        for pair in 0..WARMUP_PAIRS {
            for frozen in [pair % 2 == 0, pair % 2 != 0] {
                black_box(measure_preflight(&world, &batch, frozen));
            }
        }
        let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
        let mut current = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            for frozen in [pair % 2 == 0, pair % 2 != 0] {
                let elapsed = measure_preflight(&world, &batch, frozen);
                if frozen {
                    baseline.push(elapsed)
                } else {
                    current.push(elapsed)
                }
            }
        }
        assert_eq!(serde_json::to_value(&world).unwrap(), stored);
        assert_eq!(world.world_generation(), generation);
        let stats = world
            .ecs_frame_performance_diagnostics()
            .detached_entity_batches;
        assert_eq!(stats.commit_count, 0);
        assert_eq!(stats.rejected_preflights, 0);
        let mut restore = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
            world.reset_ecs_frame_performance_diagnostics();
            let started = Instant::now();
            let result = world.restore_detached_entity_batch(black_box(batch));
            black_box(&result);
            let elapsed = started.elapsed().as_nanos();
            result.unwrap();
            if sample >= WARMUP_PAIRS {
                restore.push(elapsed);
            }
            for root in &requested {
                assert_eq!(world.parent_of(*root), Some(depth as u64));
            }
            let stats = world
                .ecs_frame_performance_diagnostics()
                .detached_entity_batches;
            assert_eq!(stats.commit_count, 1);
            assert_eq!(stats.moved_rows, roots as u64);
            assert_eq!(stats.full_world_clone_bytes, 0);
            assert_eq!(stats.node_record_clone_bytes, 0);
            assert_eq!(stats.rollback_bytes, 0);
            batch = world
                .remove_entity_subtrees(requested.iter().copied())
                .unwrap();
            assert_eq!(batch.entity_ids().collect::<Vec<_>>(), requested);
        }
        assert_eq!(serde_json::to_value(&world).unwrap(), stored);
        eprintln!("detached_restore_union roots={roots} depth={depth} unrelated=100000 live_parent_reads={live_parent_reads} baseline_preflight_p50_p95_p99_ns={:?} current_preflight_p50_p95_p99_ns={:?} current_restore_p50_p95_p99_ns={:?} baseline_preflight_raw_ns={baseline:?} current_preflight_raw_ns={current:?} current_restore_raw_ns={restore:?}", percentiles(&baseline), percentiles(&current), percentiles(&restore));
    }
}
