use std::collections::BTreeMap;

use crate::scene::components::Name;
use crate::scene::{EntityId, NodeKind, World, WorldInspection};

fn hashes_by_entity(world: &World) -> BTreeMap<EntityId, u64> {
    world
        .inspect_hierarchy()
        .into_iter()
        .map(|row| (row.entity, row.subtree_hash))
        .collect()
}

#[test]
fn split_inspection_entries_compose_to_the_world_snapshot() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);

    let mut hierarchy_rows = world.inspect_hierarchy();
    for row in &mut hierarchy_rows {
        row.focused = row.entity == entity;
    }
    let fields = world.inspect_fields(entity);

    assert_eq!(
        WorldInspection::from_world(&world, Some(entity)),
        WorldInspection {
            generation: world.world_generation(),
            focused_entity: Some(entity),
            hierarchy_rows,
            fields,
        }
    );
    assert!(world.inspect_fields(entity + 1).is_empty());
}

#[test]
fn subtree_hash_propagates_descendant_name_changes_without_touching_unrelated_roots() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    let unrelated = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(root)).unwrap();

    let before = hashes_by_entity(&world);
    world
        .insert(child, Name("Renamed Child".to_string()))
        .unwrap();
    let after = hashes_by_entity(&world);

    assert_ne!(after[&child], before[&child]);
    assert_ne!(after[&root], before[&root]);
    assert_eq!(after[&unrelated], before[&unrelated]);
    assert_eq!(hashes_by_entity(&world), after);
}

#[test]
fn subtree_hash_tracks_parent_child_identity_without_changing_the_moved_subtree() {
    let mut world = World::empty();
    let first_parent = world.spawn_node(NodeKind::Empty);
    let second_parent = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(first_parent)).unwrap();
    let before = hashes_by_entity(&world);

    world
        .set_parent_checked(child, Some(second_parent))
        .unwrap();
    let after = hashes_by_entity(&world);

    assert_ne!(after[&first_parent], before[&first_parent]);
    assert_ne!(after[&second_parent], before[&second_parent]);
    assert_eq!(after[&child], before[&child]);
}
