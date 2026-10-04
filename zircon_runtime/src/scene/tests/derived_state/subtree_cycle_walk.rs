use super::*;
use crate::scene::components::{ActiveSelf, Name};
use crate::scene::ecs::Mut;
use crate::scene::SceneError;

fn cyclic_world(length: usize) -> (World, Vec<u64>, u64, u64, u64) {
    assert!((1..=3).contains(&length));
    let mut world = World::empty();
    let cycle = (0..length)
        .map(|_| world.spawn_node(NodeKind::Empty).unwrap())
        .collect::<Vec<_>>();
    let branch = world.spawn_node(NodeKind::Empty).unwrap();
    let tail = world.spawn_node(NodeKind::Empty).unwrap();
    let unrelated = world.spawn_node(NodeKind::Empty).unwrap();
    for edge in cycle.windows(2) {
        world.set_parent_checked(edge[0], Some(edge[1])).unwrap();
    }
    world
        .set_parent_checked(branch, Some(*cycle.last().unwrap()))
        .unwrap();
    world.set_parent_checked(tail, Some(branch)).unwrap();
    // The raw escape hatch dirties the mutation index before changing its source edge.
    assert!(world.corrupt_hierarchy_parent_for_tests(*cycle.last().unwrap(), Some(cycle[0])));
    (world, cycle, branch, tail, unrelated)
}

fn record_ids(world: &World, root: u64) -> Vec<u64> {
    world
        .subtree_records(root)
        .into_iter()
        .map(|record| record.id)
        .collect()
}

fn expected_cycle_preorder(cycle: &[u64], branch: u64, tail: u64) -> Vec<u64> {
    let mut expected = Vec::with_capacity(cycle.len() + 2);
    expected.push(cycle[0]);
    expected.extend(cycle[1..].iter().rev().copied());
    expected.extend([branch, tail]);
    expected
}

#[test]
fn subtree_walk_dirty_index_terminates_self_two_and_three_node_cycles_with_branches() {
    for length in [1, 2, 3] {
        let (world, cycle, branch, tail, unrelated) = cyclic_world(length);
        let expected = expected_cycle_preorder(&cycle, branch, tail);
        assert_eq!(record_ids(&world, cycle[0]), expected);
        assert_eq!(
            world.subtree_component_count::<Name>(cycle[0]),
            expected.len()
        );
        assert_eq!(record_ids(&world, branch), [branch, tail]);
        assert_eq!(world.subtree_component_count::<Name>(branch), 2);
        assert_eq!(record_ids(&world, unrelated), [unrelated]);
    }
}

#[test]
fn subtree_walk_current_index_terminates_self_two_and_three_node_cycles_with_branches() {
    for length in [1, 2, 3] {
        let (mut world, cycle, branch, tail, unrelated) = cyclic_world(length);
        // A valid unrelated preparation rebuilds the dirty index from the raw parents.
        let valid = world.prepare_entity_subtrees([unrelated]).unwrap();
        assert_eq!(valid.affected_entity_count(), 1);
        drop(valid);
        world.reset_ecs_frame_performance_diagnostics();
        let expected = expected_cycle_preorder(&cycle, branch, tail);
        assert_eq!(record_ids(&world, cycle[0]), expected);
        assert_eq!(
            world.subtree_component_count::<Name>(cycle[0]),
            expected.len()
        );
        assert_eq!(record_ids(&world, branch), [branch, tail]);
        assert_eq!(world.subtree_component_count::<Name>(branch), 2);
        assert_eq!(
            world
                .ecs_frame_performance_diagnostics()
                .derived_state
                .hierarchy_topology_rebuild_entities,
            0,
            "a current-index subtree walk must not rebuild the whole topology",
        );
    }
}

#[test]
fn subtree_walk_tail_start_does_not_enter_an_ancestor_cycle() {
    let (world, cycle, branch, tail, _) = cyclic_world(3);
    assert_eq!(record_ids(&world, branch), [branch, tail]);
    assert_eq!(record_ids(&world, tail), [tail]);
    assert_eq!(world.subtree_component_count::<Name>(tail), 1);
    assert_eq!(record_ids(&world, cycle[0]).len(), 5);
}

#[test]
fn subtree_walk_pending_active_query_flush_reaches_typed_parent_guard() {
    let (mut world, cycle, branch, tail, unrelated) = cyclic_world(3);
    let mut query = world.query::<Mut<'static, ActiveSelf>>();
    query.get_mut(&mut world, cycle[0]).unwrap().0 = false;
    let before = serde_json::to_value(&world).unwrap();
    let effective_generation = world.world_generation();
    let lifecycle = world.lifecycle_visibility_revision();
    world.reset_ecs_frame_performance_diagnostics();

    let error = world.prepare_entity_subtrees([cycle[0]]).unwrap_err();
    assert!(matches!(
        error,
        SceneError::HierarchyParentChainCycle { start, repeated }
            if start == cycle[0] && cycle.contains(&repeated)
    ));
    assert_eq!(world.world_generation(), effective_generation);
    assert_eq!(world.lifecycle_visibility_revision(), lifecycle);
    assert_eq!(serde_json::to_value(&world).unwrap(), before);
    assert_eq!(world.get::<ActiveSelf>(cycle[0]).unwrap().0, false);
    assert_eq!(record_ids(&world, branch), [branch, tail]);
    assert_eq!(record_ids(&world, unrelated), [unrelated]);
    let first = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(first.rejected_preflights, 1);
    assert_eq!(first.commit_count, 0);

    assert!(matches!(
        world.prepare_entity_subtrees([cycle[0]]),
        Err(SceneError::HierarchyParentChainCycle { .. })
    ));
    assert_eq!(world.world_generation(), effective_generation);
    assert_eq!(world.lifecycle_visibility_revision(), lifecycle);
    assert_eq!(serde_json::to_value(&world).unwrap(), before);
    let second = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(second.rejected_preflights, first.rejected_preflights + 1);
    assert_eq!(second.commit_count, 0);
    assert_eq!(second.moved_rows, 0);
    assert_eq!(second.lifecycle_events, 0);
}

#[test]
fn subtree_walk_valid_forest_preserves_preorder_counts_and_prepare_coverage() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let first = world.spawn_node(NodeKind::Empty).unwrap();
    let grandchild = world.spawn_node(NodeKind::Empty).unwrap();
    let second = world.spawn_node(NodeKind::Empty).unwrap();
    let unrelated = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(first, Some(root)).unwrap();
    world.set_parent_checked(grandchild, Some(first)).unwrap();
    world.set_parent_checked(second, Some(root)).unwrap();
    let expected = [root, first, grandchild, second];
    assert_eq!(record_ids(&world, root), expected);
    assert_eq!(world.subtree_component_count::<Name>(root), expected.len());
    assert_eq!(record_ids(&world, unrelated), [unrelated]);

    // A raw mutable borrow without an edge change exercises the fallback index.
    assert!(world.touch_hierarchy_parent_for_tests(grandchild));
    assert_eq!(record_ids(&world, root), expected);
    assert_eq!(world.subtree_component_count::<Name>(root), expected.len());
    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(prepared.affected_entity_count(), expected.len());
    drop(prepared);
    world.reset_ecs_frame_performance_diagnostics();
    assert_eq!(record_ids(&world, root), expected);
    assert_eq!(world.subtree_component_count::<Name>(root), expected.len());
    assert_eq!(
        world
            .ecs_frame_performance_diagnostics()
            .derived_state
            .hierarchy_topology_rebuild_entities,
        0,
    );
}
