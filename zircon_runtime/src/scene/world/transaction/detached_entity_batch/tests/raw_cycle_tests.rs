use super::*;
use crate::scene::components::Name;
use crate::scene::ecs::{Component, StorageType};
use crate::scene::inspection::SubscriptionTable;
use std::sync::{Arc, Mutex};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

fn pending_cycle(length: usize, tail: usize) -> (World, Vec<EntityId>, EntityId) {
    let mut world = World::empty();
    let chain = (0..length + tail)
        .map(|_| world.spawn_node(NodeKind::Empty).unwrap())
        .collect::<Vec<_>>();
    let valid = world.spawn_node(NodeKind::Empty).unwrap();
    for edge in chain.windows(2) {
        world.set_parent_checked(edge[0], Some(edge[1])).unwrap();
    }
    // Close the raw cycle last; checked fixture setup must not traverse it.
    assert!(world.corrupt_hierarchy_parent_for_tests(*chain.last().unwrap(), Some(chain[tail])));
    (world, chain, valid)
}

fn assert_cycle_rejected(world: &mut World, roots: &[EntityId], cycle: &[EntityId]) {
    let subscriptions = Arc::new(Mutex::new(SubscriptionTable::default()));
    subscriptions
        .lock()
        .unwrap()
        .watch(WatchRegistration::new(WatchKey::WorldStructure));
    world.attach_world_sync_subscriptions(Arc::clone(&subscriptions));
    let before = serde_json::to_value(&*world).unwrap();
    let generation = world.world_generation();
    let tick = world.read_change_tick();
    let lifecycle = world.lifecycle_visibility_revision();
    let staged = world.staged_lifecycle_events.len();
    let dirty = world.has_pending_scene_systems();
    let topology_generation = world.hierarchy_mutation_index.generation();
    let bindings = world
        .stable_entity_ids()
        .map(|id| (id, world.scene_binding_generation(id)))
        .collect::<Vec<_>>();
    world.reset_ecs_frame_performance_diagnostics();
    let derived = world.ecs_frame_performance_diagnostics().derived_state;

    for remove in [false, true] {
        let error = if remove {
            match world.remove_entity_subtrees(roots.iter().copied()) {
                Err(error) => error,
                Ok(_) => panic!("a cyclic root union must not commit even an empty batch"),
            }
        } else {
            match world.prepare_entity_subtrees(roots.iter().copied()) {
                Err(error) => error,
                Ok(_) => panic!("a cyclic root union must not issue a preparation"),
            }
        };
        let SceneError::HierarchyParentChainCycle { start, repeated } = error else {
            panic!("expected a typed parent-chain cycle, got {error:?}");
        };
        assert!(roots.contains(&start));
        assert!(cycle.contains(&repeated));
        // Stored serialization is safe on a corrupt graph and does not silently repair it.
        assert_eq!(serde_json::to_value(&*world).unwrap(), before);
        assert_eq!(world.world_generation(), generation);
        assert_eq!(world.read_change_tick(), tick);
        assert_eq!(world.lifecycle_visibility_revision(), lifecycle);
        assert_eq!(world.staged_lifecycle_events.len(), staged);
        assert_eq!(world.has_pending_scene_systems(), dirty);
        assert_eq!(
            world.hierarchy_mutation_index.generation(),
            topology_generation
        );
        assert_eq!(
            world.ecs_frame_performance_diagnostics().derived_state,
            derived
        );
        assert_eq!(
            world
                .stable_entity_ids()
                .map(|id| (id, world.scene_binding_generation(id)))
                .collect::<Vec<_>>(),
            bindings
        );
        let stats = world
            .ecs_frame_performance_diagnostics()
            .detached_entity_batches;
        assert_eq!(stats.rejected_preflights, if remove { 2 } else { 1 });
        assert_eq!(stats.commit_count, 0);
        assert_eq!(stats.moved_rows, 0);
        assert_eq!(stats.lifecycle_events, 0);
        assert_eq!(stats.generation_advances, 0);
        let table = subscriptions.lock().unwrap();
        assert_eq!(table.pending_fact_count(), 0);
        assert_eq!(table.diagnostics(), Default::default());
    }
}

#[test]
fn detached_parent_cycle_rejects_tail_before_index_rebuild() {
    let (mut world, chain, _) = pending_cycle(3, 5);
    assert_cycle_rejected(&mut world, &chain[..1], &chain[5..]);
}

#[test]
fn detached_parent_cycle_members_cannot_collapse_to_empty_success() {
    for length in [1, 2, 3] {
        let (mut world, chain, _) = pending_cycle(length, 0);
        assert_cycle_rejected(&mut world, &chain[..1], &chain);
        assert_cycle_rejected(&mut world, &chain, &chain);
    }
}

#[test]
fn detached_parent_cycle_rejects_covered_tail_and_valid_roots_atomically() {
    let (mut world, chain, valid) = pending_cycle(3, 4);
    let roots = [valid, chain[0], chain[1], chain[4], chain[0]];
    assert_cycle_rejected(&mut world, &roots, &chain[4..]);
    assert!(world.contains_entity(valid));
}

#[test]
fn detached_parent_cycle_survives_direct_persistence_before_first_flush() {
    let (world, chain, _) = pending_cycle(2, 2);
    let encoded = serde_json::to_vec(&world).unwrap();
    let mut decoded: World = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.parent_of(*chain.last().unwrap()), Some(chain[2]));
    assert_cycle_rejected(&mut decoded, &chain[..1], &chain[2..]);
}

#[test]
fn detached_parent_cycle_guard_does_not_repair_an_unrelated_cycle() {
    let (mut world, chain, valid) = pending_cycle(2, 0);
    let before = serde_json::to_value(&world).unwrap();
    let prepared = world.prepare_entity_subtrees([valid]).unwrap();
    assert_eq!(prepared.normalized_roots(), [valid]);
    assert_eq!(prepared.affected_entity_count(), 1);
    let batch = world.remove_prepared_entity_subtrees(prepared).unwrap();
    assert_eq!(world.parent_of(chain[0]), Some(chain[1]));
    assert_eq!(world.parent_of(chain[1]), Some(chain[0]));
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(serde_json::to_value(&world).unwrap(), before);
}

#[test]
fn detached_parent_cycle_preserves_existing_deferred_flush_boundary() {
    let mut world = World::empty();
    let a = world.spawn_node(NodeKind::Empty).unwrap();
    let b = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(a, Some(b)).unwrap();
    // Keep the prior valid projection current before the deferred raw edit.
    // The separate dirty-index flush-walker contract belongs to its own repair.
    drop(world.prepare_entity_subtrees([a]).unwrap());
    assert!(world.corrupt_hierarchy_parent_with_pending_mutation_for_tests(b, Some(a)));
    let before = serde_json::to_value(&world).unwrap();
    let generation = world.world_generation();
    let lifecycle = world.lifecycle_visibility_revision();
    assert!(matches!(
        world.prepare_entity_subtrees([a]),
        Err(SceneError::HierarchyParentChainCycle { .. })
    ));
    assert_eq!(serde_json::to_value(&world).unwrap(), before);
    assert_eq!(world.world_generation(), generation);
    assert_eq!(world.lifecycle_visibility_revision(), lifecycle);
    // The existing first-call flush may deliver earlier mutation notifications.
    // Once delivered, repeated rejection must have no additional observable effects.
    assert_cycle_rejected(&mut world, &[a], &[a, b]);
}

#[derive(Debug, PartialEq, Eq)]
struct TableValue(String);
impl Component for TableValue {}

#[derive(Debug, PartialEq, Eq)]
struct SparseValue(u64);
impl Component for SparseValue {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[test]
fn detached_parent_cycle_guard_preserves_valid_union_order_and_owned_rows() {
    let mut world = World::empty();
    // Stable order deliberately differs from both entity-id and hierarchy order.
    let older_child = world
        .spawn_at(
            90,
            (
                Name("child".into()),
                TableValue("owned".into()),
                SparseValue(7),
            ),
        )
        .unwrap();
    let parent = world.spawn_at(10, (Name("parent".into()),)).unwrap();
    let peer = world.spawn_at(70, (Name("peer".into()),)).unwrap();
    let survivor = world.spawn_at(20, (Name("survivor".into()),)).unwrap();
    world.set_parent_checked(older_child, Some(parent)).unwrap();
    let before = serde_json::to_value(&world).unwrap();
    let table_ticks = world.component_change_ticks::<TableValue>(older_child);
    let sparse_ticks = world.component_change_ticks::<SparseValue>(older_child);
    let prepared = world
        .prepare_entity_subtrees([peer, older_child, parent, older_child])
        .unwrap();
    assert_eq!(prepared.normalized_roots(), [parent, peer]);
    assert_eq!(prepared.affected_entity_count(), 3);
    let batch = world.remove_prepared_entity_subtrees(prepared).unwrap();
    assert_eq!(
        batch.entity_ids().collect::<Vec<_>>(),
        [older_child, parent, peer]
    );
    assert!(world.contains_entity(survivor));
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(serde_json::to_value(&world).unwrap(), before);
    assert_eq!(
        world.get::<TableValue>(older_child),
        Some(&TableValue("owned".into()))
    );
    assert_eq!(world.get::<SparseValue>(older_child), Some(&SparseValue(7)));
    assert_eq!(
        world.component_change_ticks::<TableValue>(older_child),
        table_ticks
    );
    assert_eq!(
        world.component_change_ticks::<SparseValue>(older_child),
        sparse_ticks
    );
}

#[test]
fn detached_parent_cycle_guard_keeps_missing_ancestor_and_entry_error_contracts() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(root, Some(999_999)));
    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(prepared.normalized_roots(), [root]);
    assert_eq!(prepared.affected_entity_count(), 1);
    drop(prepared);
    assert!(matches!(
        world.prepare_entity_subtrees([]),
        Err(SceneError::DetachedEntityBatchInvariant {
            reason: "detached entity root set is empty"
        })
    ));
    assert!(matches!(
        world.prepare_entity_subtrees([999_999]),
        Err(SceneError::MissingEntity {
            operation: "detach",
            entity: 999_999
        })
    ));
    assert_eq!(
        world
            .ecs_frame_performance_diagnostics()
            .detached_entity_batches
            .rejected_preflights,
        2
    );
    world.world_generation = world.world_generation.advanced_by(u64::MAX);
    assert!(matches!(
        world.prepare_entity_subtrees([]),
        Err(SceneError::DetachedEntityPreparationGenerationExhausted)
    ));
}
