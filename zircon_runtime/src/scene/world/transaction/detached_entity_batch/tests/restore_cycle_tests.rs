use super::*;
use crate::scene::components::Name;
use crate::scene::ecs::{Component, StorageType};
use crate::scene::inspection::SubscriptionTable;
use std::sync::{Arc, Mutex};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

#[derive(Debug, PartialEq, Eq)]
struct OwnedTable(Box<str>);
impl Component for OwnedTable {}

#[derive(Debug, PartialEq, Eq)]
struct OwnedSparse(u64);
impl Component for OwnedSparse {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[derive(Debug)]
struct RestoreProbe(u64);

fn rejected_restore_preserves_world(
    world: &mut World,
    batch: DetachedEntityBatch,
    check_error: impl FnOnce(&SceneError),
) -> DetachedEntityBatch {
    let subscriptions = Arc::new(Mutex::new(SubscriptionTable::default()));
    subscriptions
        .lock()
        .unwrap()
        .watch(WatchRegistration::new(WatchKey::WorldStructure));
    world.attach_world_sync_subscriptions(Arc::clone(&subscriptions));
    world.reset_ecs_frame_performance_diagnostics();
    let stored = serde_json::to_value(&*world).unwrap();
    let generation = world.world_generation();
    let pending_mutations = world.derived_state_dirty.pending_component_mutation_count();
    let change_tick = world.read_change_tick();
    let lifecycle = world.lifecycle_visibility_revision();
    let staged_events = world.staged_lifecycle_events.len();
    let active_camera = world.active_camera();
    let pending_scene_systems = world.has_pending_scene_systems();
    let topology_generation = world.hierarchy_mutation_index.generation();
    let bindings = world
        .stable_entity_ids()
        .map(|entity| (entity, world.scene_binding_generation(entity)))
        .collect::<Vec<_>>();
    let derived_diagnostics = world.ecs_frame_performance_diagnostics().derived_state;

    let rejected = world.restore_detached_entity_batch(batch).unwrap_err();
    check_error(rejected.error());
    assert_eq!(serde_json::to_value(&*world).unwrap(), stored);
    assert_eq!(world.world_generation(), generation);
    assert_eq!(
        world.derived_state_dirty.pending_component_mutation_count(),
        pending_mutations
    );
    assert_eq!(world.read_change_tick(), change_tick);
    assert_eq!(world.lifecycle_visibility_revision(), lifecycle);
    assert_eq!(world.staged_lifecycle_events.len(), staged_events);
    assert_eq!(world.active_camera(), active_camera);
    assert_eq!(world.has_pending_scene_systems(), pending_scene_systems);
    assert_eq!(
        world.hierarchy_mutation_index.generation(),
        topology_generation
    );
    assert_eq!(
        world
            .stable_entity_ids()
            .map(|entity| (entity, world.scene_binding_generation(entity)))
            .collect::<Vec<_>>(),
        bindings
    );
    assert_eq!(
        world.ecs_frame_performance_diagnostics().derived_state,
        derived_diagnostics
    );
    let stats = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(stats.rejected_preflights, 1);
    assert_eq!(stats.commit_count, 0);
    assert_eq!(stats.moved_rows, 0);
    assert_eq!(stats.lifecycle_events, 0);
    assert_eq!(stats.generation_advances, 0);
    assert_eq!(stats.full_world_clone_bytes, 0);
    assert_eq!(stats.node_record_clone_bytes, 0);
    assert_eq!(stats.rollback_bytes, 0);
    let watched = subscriptions.lock().unwrap();
    assert_eq!(watched.pending_fact_count(), 0);
    assert_eq!(watched.diagnostics(), Default::default());
    rejected.into_parts().1
}

#[test]
fn restore_union_cycle_rejects_and_returns_owned_batch_for_retry() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let restored = world
        .spawn((
            Name("restored".into()),
            OwnedTable("table payload".into()),
            OwnedSparse(17),
        ))
        .unwrap();
    world.set_parent_checked(restored, Some(parent)).unwrap();
    world
        .set_dynamic_component(
            restored,
            "test.restore_cycle_payload",
            serde_json::json!({"v": 9}),
        )
        .unwrap();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let for_callback = Arc::clone(&observed);
    world.observe_entity_event::<RestoreProbe>(restored, move |_world, entity, event| {
        for_callback.lock().unwrap().push((entity, event.0));
    });
    let table_ticks = world
        .component_change_ticks::<OwnedTable>(restored)
        .unwrap();
    let sparse_ticks = world
        .component_change_ticks::<OwnedSparse>(restored)
        .unwrap();
    let batch = world.remove_entity_recursive(restored).unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(parent, Some(restored)));

    let batch = rejected_restore_preserves_world(&mut world, batch, |error| {
        assert_eq!(
            error,
            &SceneError::HierarchyCycle {
                child: restored,
                parent,
            }
        );
    });
    assert_eq!(batch.entity_ids().collect::<Vec<_>>(), [restored]);
    world.set_parent_checked(parent, None).unwrap();
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.parent_of(restored), Some(parent));
    assert_eq!(
        world.get::<OwnedTable>(restored),
        Some(&OwnedTable("table payload".into()))
    );
    assert_eq!(world.get::<OwnedSparse>(restored), Some(&OwnedSparse(17)));
    assert_eq!(
        world.component_change_ticks::<OwnedTable>(restored),
        Some(table_ticks)
    );
    assert_eq!(
        world.component_change_ticks::<OwnedSparse>(restored),
        Some(sparse_ticks)
    );
    assert_eq!(
        world.dynamic_component(restored, "test.restore_cycle_payload"),
        Some(&serde_json::json!({"v": 9}))
    );
    world.trigger_entity_event(restored, RestoreProbe(41));
    assert_eq!(*observed.lock().unwrap(), vec![(restored, 41)]);
}

#[test]
fn restore_union_cycle_rejects_three_edges_across_restored_descendant() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let restored = world.spawn_node(NodeKind::Empty).unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(restored, Some(parent)).unwrap();
    world.set_parent_checked(child, Some(restored)).unwrap();
    let batch = world.remove_entity_recursive(restored).unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(parent, Some(child)));
    let batch = rejected_restore_preserves_world(&mut world, batch, |error| {
        assert_eq!(
            error,
            &SceneError::HierarchyCycle {
                child: restored,
                parent,
            }
        );
    });
    assert_eq!(batch.entity_ids().collect::<Vec<_>>(), [restored, child]);
    world.set_parent_checked(parent, None).unwrap();
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.parent_of(restored), Some(parent));
    assert_eq!(world.parent_of(child), Some(restored));
}

#[test]
fn restore_reachable_live_cycle_has_existing_chain_diagnostic() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let other = world.spawn_node(NodeKind::Empty).unwrap();
    let restored = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(restored, Some(parent)).unwrap();
    let batch = world.remove_entity_recursive(restored).unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(parent, Some(other)));
    assert!(world.corrupt_hierarchy_parent_for_tests(other, Some(parent)));
    let _batch = rejected_restore_preserves_world(&mut world, batch, |error| {
        assert_eq!(
            error,
            &SceneError::HierarchyParentChainCycle {
                start: restored,
                repeated: parent,
            }
        );
    });
}

#[test]
fn restore_union_cycle_finds_a_batch_edge_when_the_repeated_node_is_live() {
    let mut world = World::empty();
    let live_parent = world.spawn_node(NodeKind::Empty).unwrap();
    let earlier_batch_root = world.spawn_node(NodeKind::Empty).unwrap();
    let cycle_batch_root = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_parent_checked(earlier_batch_root, Some(live_parent))
        .unwrap();
    world
        .set_parent_checked(cycle_batch_root, Some(live_parent))
        .unwrap();
    let batch = world
        .remove_entity_subtrees([earlier_batch_root, cycle_batch_root])
        .unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(live_parent, Some(cycle_batch_root)));
    let _batch = rejected_restore_preserves_world(&mut world, batch, |error| {
        assert_eq!(
            error,
            &SceneError::HierarchyCycle {
                child: cycle_batch_root,
                parent: live_parent,
            }
        );
    });
}

#[test]
fn restore_does_not_scan_or_repair_an_unrelated_live_cycle() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let restored = world.spawn_node(NodeKind::Empty).unwrap();
    let unrelated_a = world.spawn_node(NodeKind::Empty).unwrap();
    let unrelated_b = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(restored, Some(parent)).unwrap();
    let batch = world.remove_entity_recursive(restored).unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(unrelated_a, Some(unrelated_b)));
    assert!(world.corrupt_hierarchy_parent_for_tests(unrelated_b, Some(unrelated_a)));
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.parent_of(restored), Some(parent));
    assert_eq!(world.parent_of(unrelated_a), Some(unrelated_b));
    assert_eq!(world.parent_of(unrelated_b), Some(unrelated_a));
}

#[test]
fn restore_preserves_the_existing_missing_live_ancestor_semantics() {
    let mut world = World::empty();
    let live_parent = world.spawn_node(NodeKind::Empty).unwrap();
    let restored = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_parent_checked(restored, Some(live_parent))
        .unwrap();
    let batch = world.remove_entity_recursive(restored).unwrap();
    let missing_ancestor = 999_999;
    assert!(world.corrupt_hierarchy_parent_for_tests(live_parent, Some(missing_ancestor)));
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.parent_of(restored), Some(live_parent));
    assert_eq!(world.parent_of(live_parent), Some(missing_ancestor));
    assert!(!world.contains_entity(missing_ancestor));
}

#[test]
fn restore_rejection_preserves_unflushed_query_mutation_and_effective_generation() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let restored = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(restored, Some(parent)).unwrap();
    let batch = world.remove_entity_recursive(restored).unwrap();
    assert!(world.corrupt_hierarchy_parent_with_pending_mutation_for_tests(parent, Some(restored)));
    assert_eq!(
        world.derived_state_dirty.pending_component_mutation_count(),
        1
    );
    let _batch = rejected_restore_preserves_world(&mut world, batch, |error| {
        assert_eq!(
            error,
            &SceneError::HierarchyCycle {
                child: restored,
                parent,
            }
        );
    });
    assert_eq!(
        world.derived_state_dirty.pending_component_mutation_count(),
        1
    );
}

#[test]
fn restore_missing_parent_precedes_another_entrys_union_cycle() {
    let mut world = World::empty();
    let missing_parent = world.spawn_node(NodeKind::Empty).unwrap();
    let cycle_parent = world.spawn_node(NodeKind::Empty).unwrap();
    let first = world.spawn_node(NodeKind::Empty).unwrap();
    let second = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_parent_checked(first, Some(missing_parent))
        .unwrap();
    world
        .set_parent_checked(second, Some(cycle_parent))
        .unwrap();
    let batch = world.remove_entity_subtrees([first, second]).unwrap();
    world.remove_entity(missing_parent).unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(cycle_parent, Some(second)));
    let _batch = rejected_restore_preserves_world(&mut world, batch, |error| {
        assert_eq!(
            error,
            &SceneError::MissingParent {
                child: first,
                parent: missing_parent,
            }
        );
    });
}

#[test]
fn restore_duplicate_entity_precedes_a_union_cycle() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let restored = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(restored, Some(parent)).unwrap();
    let batch = world.remove_entity_recursive(restored).unwrap();
    world
        .spawn_at(restored, (Name("conflict".into()),))
        .unwrap();
    assert!(world.corrupt_hierarchy_parent_for_tests(parent, Some(restored)));
    let _batch = rejected_restore_preserves_world(&mut world, batch, |error| {
        assert_eq!(error, &SceneError::DuplicateEntity { entity: restored });
    });
}

#[test]
fn restore_valid_multi_root_union_keeps_stable_order_after_live_reparent() {
    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let other_parent = world.spawn_node(NodeKind::Empty).unwrap();
    let first = world.spawn_at(90, (OwnedTable("first".into()),)).unwrap();
    let second = world.spawn_at(30, (OwnedSparse(2),)).unwrap();
    world.set_parent_checked(first, Some(parent)).unwrap();
    world.set_parent_checked(second, Some(parent)).unwrap();
    let before_ids = world.stable_entity_ids().collect::<Vec<_>>();
    let batch = world.remove_entity_subtrees([first, second]).unwrap();
    world
        .set_parent_checked(parent, Some(other_parent))
        .unwrap();
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.stable_entity_ids().collect::<Vec<_>>(), before_ids);
    assert_eq!(world.parent_of(first), Some(parent));
    assert_eq!(world.parent_of(second), Some(parent));
    assert_eq!(
        world.get::<OwnedTable>(first),
        Some(&OwnedTable("first".into()))
    );
    assert_eq!(world.get::<OwnedSparse>(second), Some(&OwnedSparse(2)));
}
