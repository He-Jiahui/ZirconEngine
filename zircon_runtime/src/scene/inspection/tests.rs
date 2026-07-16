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

fn world_with_serialized_parents(world: &World, parents: &[(EntityId, Option<EntityId>)]) -> World {
    let mut encoded = serde_json::to_value(world).unwrap();
    let hierarchy = encoded
        .get_mut("hierarchy")
        .and_then(serde_json::Value::as_object_mut)
        .expect("serialized world hierarchy should be an object");
    for (entity, parent) in parents {
        let row = hierarchy
            .get_mut(&entity.to_string())
            .and_then(serde_json::Value::as_object_mut)
            .expect("serialized hierarchy row should be an object");
        row.insert("parent".to_string(), serde_json::to_value(parent).unwrap());
    }
    serde_json::from_value(encoded).unwrap()
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

#[test]
fn subtree_hash_encodes_cycle_edges_even_when_the_child_was_already_visited() {
    let mut source = World::empty();
    let first = source.spawn_node(NodeKind::Empty);
    let second = source.spawn_node(NodeKind::Empty);
    let missing_parent = second + 100;

    let cycle =
        world_with_serialized_parents(&source, &[(first, Some(second)), (second, Some(first))]);
    let broken_cycle = world_with_serialized_parents(
        &source,
        &[(first, Some(missing_parent)), (second, Some(first))],
    );

    assert_eq!(
        cycle
            .inspect_hierarchy()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_eq!(
        broken_cycle
            .inspect_hierarchy()
            .iter()
            .map(|row| row.entity)
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_ne!(hashes_by_entity(&cycle), hashes_by_entity(&broken_cycle));
}

#[test]
fn deep_hierarchy_inspection_is_iterative_and_deterministic() {
    const DEPTH: usize = 5_000;

    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let mut parent = root;
    for _ in 1..DEPTH {
        let child = world.spawn_node(NodeKind::Empty);
        world.set_parent_checked(child, Some(parent)).unwrap();
        parent = child;
    }

    let first = world.inspect_hierarchy();
    let second = world.inspect_hierarchy();

    assert_eq!(first.len(), DEPTH);
    assert_eq!(first, second);
    assert_eq!(first.first().map(|row| row.depth), Some(0));
    assert_eq!(first.last().map(|row| row.depth), Some((DEPTH - 1) as u32));
}
