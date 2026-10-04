//! Release-only paired profile on valid forests. Frozen bodies are from the Q
//! derived_state.rs preimage (SHA256 4c5fcea969cd4e562eb219af437e0cf59517268af4a572d0a7baac12559bfd55).
use super::*;
use std::hint::black_box;
use std::time::Instant;

const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;
const CHAIN_DEPTH: usize = 4_096;
const UNRELATED_ENTITIES: usize = 100_000;

impl World {
    // Full public wrapper plus complete frozen indexed/fallback collection bodies.
    fn frozen_subtree_records(&self, entity: EntityId) -> Vec<NodeRecord> {
        let mut records = Vec::new();
        self.frozen_collect_subtree_records(entity, &mut records);
        records
    }

    fn frozen_collect_subtree_records(&self, entity: EntityId, records: &mut Vec<NodeRecord>) {
        if self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            let mut stack = vec![entity];
            while let Some(current) = stack.pop() {
                let Some(record) = self.node_record(current) else {
                    continue;
                };
                records.push(record);
                stack.extend(self.hierarchy_mutation_index.children_of(current).rev());
            }
            return;
        }
        let traversal = self.hierarchy_traversal_index();
        self.frozen_collect_subtree_records_with_traversal(entity, records, &traversal);
    }

    fn frozen_collect_subtree_records_with_traversal(
        &self,
        entity: EntityId,
        records: &mut Vec<NodeRecord>,
        traversal: &HierarchyTraversalIndex,
    ) {
        let mut stack = vec![entity];
        while let Some(current) = stack.pop() {
            let Some(record) = self.node_record(current) else {
                continue;
            };
            records.push(record);
            stack.extend(traversal.children_of(current).iter().rev().copied());
        }
    }

    // Complete frozen public count body, including both index branches.
    fn frozen_subtree_component_count<T>(&self, root: EntityId) -> usize
    where
        T: Component,
    {
        let Some(component_id) = self.registered_component_id::<T>() else {
            return 0;
        };
        if !self.contains_entity(root) {
            return 0;
        }

        let mut count = 0;
        let mut stack = vec![root];
        if self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len())
        {
            while let Some(entity) = stack.pop() {
                count += usize::from(self.contains_component_id(entity, component_id));
                stack.extend(self.hierarchy_mutation_index.children_of(entity).rev());
            }
            return count;
        }

        let traversal = self.hierarchy_traversal_index();
        while let Some(entity) = stack.pop() {
            count += usize::from(self.contains_component_id(entity, component_id));
            stack.extend(traversal.children_of(entity).iter().rev().copied());
        }
        count
    }
}

fn legal_forest(branches: usize) -> (World, EntityId, usize) {
    let mut seed = World::empty();
    let seed_id = seed.spawn_node(NodeKind::Empty).unwrap().to_string();
    let mut document = serde_json::to_value(&seed).unwrap();
    let fields = document.as_object_mut().unwrap();
    let affected = CHAIN_DEPTH + branches;
    let count = affected + UNRELATED_ENTITIES;
    // Persistent rows construct a real World without quadratic checked setup.
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
        let parent = if entity == 1 {
            None
        } else if entity <= CHAIN_DEPTH {
            Some(entity as u64 - 1)
        } else {
            Some(CHAIN_DEPTH as u64)
        };
        hierarchy.insert(entity.to_string(), serde_json::json!({"parent": parent}));
    }
    let mut world: World = serde_json::from_value(document).unwrap();
    world.ensure_hierarchy_mutation_index_current();
    (world, 1, affected)
}

fn percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let at = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (at(50), at(95), at(99))
}

fn measure_records(world: &World, root: EntityId, frozen: bool, expected: &[NodeRecord]) -> u128 {
    let started = Instant::now();
    let actual = if frozen {
        world.frozen_subtree_records(black_box(root))
    } else {
        world.subtree_records(black_box(root))
    };
    black_box(&actual);
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(actual, expected);
    drop(actual);
    elapsed
}

fn measure_count(world: &World, root: EntityId, frozen: bool, expected: usize) -> u128 {
    let started = Instant::now();
    let actual = if frozen {
        world.frozen_subtree_component_count::<Name>(black_box(root))
    } else {
        world.subtree_component_count::<Name>(black_box(root))
    };
    black_box(actual);
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(actual, expected);
    elapsed
}

#[test]
#[ignore = "Release paired public subtree API profile; run under managed Runtime62 validation"]
fn runtime62_subtree_cycle_guard_public_api_release_profile() {
    assert!(!cfg!(debug_assertions), "run the paired profile in Release");
    for branches in [2, 128] {
        let (world, root, affected) = legal_forest(branches);
        let expected_records = world.frozen_subtree_records(root);
        assert_eq!(expected_records.len(), affected);
        assert_eq!(world.frozen_subtree_component_count::<Name>(root), affected);
        assert_eq!(world.subtree_records(root), expected_records);
        assert_eq!(world.subtree_component_count::<Name>(root), affected);
        let generation = world.world_generation();

        for operation in ["records", "count"] {
            let measure = |frozen| match operation {
                "records" => measure_records(&world, root, frozen, &expected_records),
                "count" => measure_count(&world, root, frozen, affected),
                _ => unreachable!(),
            };
            for pair in 0..WARMUP_PAIRS {
                for frozen in [pair % 2 == 0, pair % 2 != 0] {
                    black_box(measure(frozen));
                }
            }
            let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
            let mut current = Vec::with_capacity(SAMPLE_PAIRS);
            for pair in 0..SAMPLE_PAIRS {
                for frozen in [pair % 2 == 0, pair % 2 != 0] {
                    let elapsed = measure(frozen);
                    if frozen {
                        baseline.push(elapsed);
                    } else {
                        current.push(elapsed);
                    }
                }
            }
            assert_eq!(world.world_generation(), generation);
            assert_eq!(world.subtree_component_count::<Name>(root), affected);
            eprintln!(
                "subtree_cycle_guard operation={operation} branches={branches} depth={CHAIN_DEPTH} unrelated={UNRELATED_ENTITIES} visited={affected} baseline_p50_p95_p99_ns={:?} current_p50_p95_p99_ns={:?} baseline_raw_ns={baseline:?} current_raw_ns={current:?}",
                percentiles(&baseline),
                percentiles(&current)
            );
        }
    }
}
