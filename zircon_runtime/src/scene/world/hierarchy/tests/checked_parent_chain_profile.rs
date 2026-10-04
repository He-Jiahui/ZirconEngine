//! Paired real checked-reparent timings; the frozen path is called only on valid forests.

use super::*;
use crate::scene::NodeKind;
use std::hint::black_box;
use std::time::Instant;

const DEPTHS: [usize; 4] = [1, 32, 1_000, 100_000];
const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;

// The complete preimage methods are retained, including existing fact publication.
// Only test-local visibility, names and the mutual call differ from the frozen source.
impl World {
    fn frozen_is_descendant(&self, entity: EntityId, ancestor: EntityId) -> bool {
        let mut cursor = Some(entity);
        while let Some(current) = cursor {
            if current == ancestor {
                return true;
            }
            cursor = self.parent_of(current);
        }
        false
    }

    fn frozen_set_parent_checked(
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
            if self.frozen_is_descendant(parent, child) {
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
        self.insert_checked_hierarchy(child, Hierarchy { parent })?;
        self.record_world_fact(WorldFact::Reparented {
            entity: child,
            new_parent: parent,
        });
        Ok(true)
    }
}

fn valid_chain_document(depth: usize) -> serde_json::Value {
    let mut seed = World::empty();
    let seed_entity = seed.spawn_node(NodeKind::Empty).unwrap();
    let seed_key = seed_entity.to_string();
    let mut document = serde_json::to_value(&seed).unwrap();
    let fields = document.as_object_mut().unwrap();
    let count = depth + 1;
    // Build persisted rows directly, avoiding O(depth^2) checked setup mutations.
    // This uses the real World deserializer and its real component/topology projection.
    for value in fields.values_mut() {
        if let Some(rows) = value.as_object_mut() {
            if let Some(template) = rows.get(&seed_key).cloned() {
                rows.clear();
                for entity in 1..=count {
                    rows.insert(entity.to_string(), template.clone());
                }
            }
        }
    }
    fields.insert(
        "entities".to_string(),
        serde_json::json!((1..=count as u64).collect::<Vec<_>>()),
    );
    fields.insert("next_id".to_string(), serde_json::json!(count as u64 + 1));
    let hierarchy = fields
        .get_mut("hierarchy")
        .unwrap()
        .as_object_mut()
        .unwrap();
    for entity in 1..=count {
        let parent = if entity > 1 && entity <= depth {
            Some(entity as u64 - 1)
        } else {
            None
        };
        hierarchy.insert(entity.to_string(), serde_json::json!({ "parent": parent }));
    }
    document
}

fn measure_reparent(world: &mut World, depth: usize, frozen: bool) -> u128 {
    let parent = depth as u64;
    let child = parent + 1;
    assert_eq!(world.parent_of(child), None);
    let started = Instant::now();
    let result = if frozen {
        world.frozen_set_parent_checked(black_box(child), black_box(Some(parent)))
    } else {
        world.set_parent_checked(black_box(child), black_box(Some(parent)))
    };
    black_box(&result);
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(result, Ok(true));
    assert_eq!(world.parent_of(child), Some(parent));
    // Restore outside the timed interval. Both paths use their own complete public-operation body.
    let restored = if frozen {
        world.frozen_set_parent_checked(child, None)
    } else {
        world.set_parent_checked(child, None)
    };
    assert_eq!(restored, Ok(true));
    assert_eq!(world.parent_of(child), None);
    elapsed
}

fn percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let at = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (at(50), at(95), at(99))
}

#[test]
#[ignore = "Release paired checked-reparent profile; schedule in a managed batch"]
fn runtime62_checked_parent_chain_release_profile() {
    assert!(!cfg!(debug_assertions), "this profile requires Release");
    for depth in DEPTHS {
        let document = valid_chain_document(depth);
        let mut frozen: World = serde_json::from_value(document.clone()).unwrap();
        let mut current: World = serde_json::from_value(document).unwrap();
        for pair in 0..WARMUP_PAIRS {
            if pair % 2 == 0 {
                measure_reparent(&mut frozen, depth, true);
                measure_reparent(&mut current, depth, false);
            } else {
                measure_reparent(&mut current, depth, false);
                measure_reparent(&mut frozen, depth, true);
            }
        }
        let mut frozen_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut current_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                frozen_ns.push(measure_reparent(&mut frozen, depth, true));
                current_ns.push(measure_reparent(&mut current, depth, false));
            } else {
                current_ns.push(measure_reparent(&mut current, depth, false));
                frozen_ns.push(measure_reparent(&mut frozen, depth, true));
            }
        }
        assert_eq!(
            serde_json::to_value(&current).unwrap(),
            serde_json::to_value(&frozen).unwrap()
        );
        assert_eq!(current.world_generation(), frozen.world_generation());
        assert_eq!(
            current.ecs_frame_performance_diagnostics().derived_state,
            frozen.ecs_frame_performance_diagnostics().derived_state
        );
        let old = percentiles(&frozen_ns);
        let new = percentiles(&current_ns);
        println!(
            "RUNTIME62_CHECKED_PARENT_CHAIN_BENCH_V1 depth={depth} samples={SAMPLE_PAIRS} frozen_ns={frozen_ns:?} current_ns={current_ns:?} frozen_p50_ns={} frozen_p95_ns={} frozen_p99_ns={} current_p50_ns={} current_p95_ns={} current_p99_ns={}",
            old.0, old.1, old.2, new.0, new.1, new.2,
        );
    }
}
