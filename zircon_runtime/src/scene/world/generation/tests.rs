use crate::scene::{NodeKind, World};

#[test]
fn world_generation_advances_only_for_successful_structural_mutations() {
    let mut world = World::empty();
    assert_eq!(world.world_generation(), 0);

    let root = world.spawn_node(NodeKind::Empty);
    assert_eq!(world.world_generation(), 1);

    let child = world.spawn_node(NodeKind::Empty);
    assert_eq!(world.world_generation(), 2);

    assert!(!world.set_parent_checked(child, None).unwrap());
    assert_eq!(world.world_generation(), 2);

    assert!(world.set_parent_checked(child, Some(root)).unwrap());
    assert_eq!(world.world_generation(), 3);

    assert!(!world.remove_entity(u64::MAX));
    assert_eq!(world.world_generation(), 3);

    assert!(world.remove_entity(child));
    assert_eq!(world.world_generation(), 4);
}

#[test]
fn explicit_entity_spawn_advances_generation_exactly_once() {
    let mut world = World::empty();

    assert!(world.spawn_empty_at(42));
    assert_eq!(world.world_generation(), 1);

    assert!(!world.spawn_empty_at(42));
    assert_eq!(world.world_generation(), 1);
}

#[test]
fn imported_node_record_spawn_advances_generation_exactly_once() {
    let mut source = World::empty();
    let source_entity = source.spawn_node(NodeKind::Empty);
    let record = source.node_record(source_entity).unwrap();
    let mut target = World::empty();

    target.insert_node_record(record).unwrap();

    assert_eq!(target.world_generation(), 1);
}

#[test]
fn component_replacement_advances_the_query_generation_revision() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    let before_rename = world.world_generation();

    assert!(world.rename_node(entity, "Renamed").unwrap());
    assert_eq!(world.world_generation(), before_rename + 1);

    let after_rename = world.world_generation();
    assert!(!world.rename_node(entity, "Renamed").unwrap());
    assert_eq!(world.world_generation(), after_rename);
}

#[test]
fn world_generation_is_runtime_state_not_persisted_scene_data() {
    let mut world = World::empty();
    world.spawn_node(NodeKind::Empty);
    assert_eq!(world.world_generation(), 1);

    let encoded = serde_json::to_value(&world).unwrap();
    assert!(encoded.get("world_generation").is_none());

    let restored: World = serde_json::from_value(encoded).unwrap();
    assert_eq!(restored.world_generation(), 0);
}
