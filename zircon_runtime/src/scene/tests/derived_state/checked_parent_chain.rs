//! Checked reparent must reject a corrupt pending parent chain before publishing a mutation.

use std::sync::{Arc, Mutex};

use crate::scene::components::Mobility;
use crate::scene::inspection::SubscriptionTable;
use crate::scene::{EntityId, NodeKind, SceneError, World};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

fn empty_nodes(count: usize) -> (World, Vec<EntityId>) {
    let mut world = World::empty();
    let nodes = (0..count)
        .map(|_| world.spawn_node(NodeKind::Empty).unwrap())
        .collect();
    (world, nodes)
}

fn pending_cycle(cycle_len: usize, tail_len: usize) -> (World, Vec<EntityId>, EntityId) {
    let (mut world, mut chain) = empty_nodes(cycle_len + tail_len + 1);
    let requested_child = chain.pop().unwrap();
    for edge in chain.windows(2) {
        world.set_parent_checked(edge[0], Some(edge[1])).unwrap();
    }
    // Close the cycle last: no fixture mutation may walk the now-corrupt graph.
    assert!(world.corrupt_hierarchy_parent_for_tests(*chain.last().unwrap(), Some(chain[tail_len])));
    (world, chain, requested_child)
}

fn assert_cycle_rejected_without_mutation(
    world: &mut World,
    chain: &[EntityId],
    tail_len: usize,
    child: EntityId,
) {
    let subscriptions = Arc::new(Mutex::new(SubscriptionTable::default()));
    subscriptions
        .lock()
        .unwrap()
        .watch(WatchRegistration::new(WatchKey::WorldStructure));
    world.attach_world_sync_subscriptions(Arc::clone(&subscriptions));

    // These snapshots only read stored state; none normalizes or flushes hierarchy validity.
    let before = serde_json::to_value(&*world).unwrap();
    let generation = world.world_generation();
    let change_tick = world.read_change_tick();
    let dirty = world.has_pending_scene_systems();
    let derived = world.ecs_frame_performance_diagnostics().derived_state;
    let binding_generations = chain
        .iter()
        .map(|entity| world.scene_binding_generation(*entity))
        .collect::<Vec<_>>();

    let error = world.set_parent_checked(child, Some(chain[0])).unwrap_err();

    let SceneError::HierarchyParentChainCycle { start, repeated } = error else {
        panic!("expected a pre-existing parent-chain cycle, got {error:?}");
    };
    assert_eq!(start, chain[0]);
    assert!(chain[tail_len..].contains(&repeated));
    assert_eq!(serde_json::to_value(&*world).unwrap(), before);
    assert_eq!(world.parent_of(child), None);
    assert_eq!(world.world_generation(), generation);
    assert_eq!(world.read_change_tick(), change_tick);
    assert_eq!(world.has_pending_scene_systems(), dirty);
    assert_eq!(
        world.ecs_frame_performance_diagnostics().derived_state,
        derived
    );
    assert_eq!(
        chain
            .iter()
            .map(|entity| world.scene_binding_generation(*entity))
            .collect::<Vec<_>>(),
        binding_generations
    );
    let table = subscriptions.lock().unwrap();
    assert_eq!(table.pending_fact_count(), 0);
    assert_eq!(table.diagnostics(), Default::default());
}

#[test]
fn checked_parent_chain_rejects_pending_two_node_cycle_without_mutation() {
    let (mut world, chain, child) = pending_cycle(2, 0);
    assert_cycle_rejected_without_mutation(&mut world, &chain, 0, child);
}

#[test]
fn checked_parent_chain_rejects_pending_three_node_cycle_without_mutation() {
    let (mut world, chain, child) = pending_cycle(3, 0);
    assert_cycle_rejected_without_mutation(&mut world, &chain, 0, child);
}

#[test]
fn checked_parent_chain_rejects_pending_self_cycle_without_mutation() {
    let (mut world, chain, child) = pending_cycle(1, 0);
    assert_cycle_rejected_without_mutation(&mut world, &chain, 0, child);
}

#[test]
fn checked_parent_chain_rejects_tail_entering_a_cycle_without_mutation() {
    let (mut world, chain, child) = pending_cycle(3, 5);
    assert_cycle_rejected_without_mutation(&mut world, &chain, 5, child);
}

#[test]
fn checked_parent_chain_rejects_direct_serde_cycle_before_first_flush() {
    let (world, chain, child) = pending_cycle(2, 2);
    let encoded = serde_json::to_vec(&world).unwrap();
    let mut restored: World = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(restored.parent_of(*chain.last().unwrap()), Some(chain[2]));
    assert!(restored.has_pending_scene_systems());
    assert_cycle_rejected_without_mutation(&mut restored, &chain, 2, child);
}

#[test]
fn checked_parent_chain_keeps_ancestor_hit_a_prospective_cycle_error() {
    let (mut world, chain, _) = pending_cycle(3, 0);
    let before = serde_json::to_value(&world).unwrap();
    let generation = world.world_generation();

    assert_eq!(
        world.set_parent_checked(chain[1], Some(chain[0])),
        Err(SceneError::HierarchyCycle {
            child: chain[1],
            parent: chain[0],
        })
    );
    assert_eq!(serde_json::to_value(&world).unwrap(), before);
    assert_eq!(world.world_generation(), generation);
}

#[test]
fn checked_parent_chain_preserves_valid_reparent_noop_and_detach() {
    let (mut world, nodes) = empty_nodes(4);
    world.set_parent_checked(nodes[0], Some(nodes[1])).unwrap();
    world.set_parent_checked(nodes[1], Some(nodes[2])).unwrap();
    assert_eq!(world.set_parent_checked(nodes[3], Some(nodes[0])), Ok(true));
    assert_eq!(world.parent_of(nodes[3]), Some(nodes[0]));
    let generation = world.world_generation();
    assert_eq!(
        world.set_parent_checked(nodes[3], Some(nodes[0])),
        Ok(false)
    );
    assert_eq!(world.world_generation(), generation);
    assert_eq!(world.set_parent_checked(nodes[3], None), Ok(true));
    assert_eq!(world.parent_of(nodes[3]), None);
}

#[test]
fn checked_parent_chain_preserves_existing_error_priority() {
    let (mut world, nodes) = empty_nodes(3);
    let missing = u64::MAX;
    assert_eq!(
        world.set_parent_checked(missing, Some(missing)),
        Err(SceneError::MissingEntity {
            operation: "reparent",
            entity: missing,
        })
    );
    assert_eq!(
        world.set_parent_checked(nodes[0], Some(nodes[0])),
        Err(SceneError::EntityCannotParentItself { entity: nodes[0] })
    );
    assert_eq!(
        world.set_parent_checked(nodes[0], Some(missing)),
        Err(SceneError::MissingParent {
            child: nodes[0],
            parent: missing,
        })
    );
    world.set_parent_checked(nodes[0], Some(nodes[1])).unwrap();
    assert_eq!(
        world.set_parent_checked(nodes[1], Some(nodes[0])),
        Err(SceneError::HierarchyCycle {
            child: nodes[1],
            parent: nodes[0],
        })
    );
    world.set_mobility(nodes[2], Mobility::Static).unwrap();
    assert_eq!(
        world.set_parent_checked(nodes[2], Some(nodes[1])),
        Err(SceneError::StaticReparentMutation { entity: nodes[2] })
    );
}
