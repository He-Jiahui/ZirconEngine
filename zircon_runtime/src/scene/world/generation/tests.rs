use std::sync::{Arc, Mutex};

use crate::scene::components::{Mobility, Name};
use crate::scene::ecs::LifecycleEventKind;
use crate::scene::{NodeKind, SceneError, World};

#[test]
fn world_generation_advances_only_for_successful_structural_mutations() {
    let mut world = World::empty();
    assert_eq!(world.world_generation(), 0);

    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    assert_eq!(world.world_generation(), 1);

    let child = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    assert_eq!(world.world_generation(), 2);

    assert!(!world.set_parent_checked(child, None).unwrap());
    assert_eq!(world.world_generation(), 2);

    assert!(world.set_parent_checked(child, Some(root)).unwrap());
    assert_eq!(world.world_generation(), 3);

    assert!(matches!(
        world.remove_entity(u64::MAX),
        Err(crate::scene::SceneError::MissingEntity { entity, .. }) if entity == u64::MAX
    ));
    assert_eq!(world.world_generation(), 3);

    world.remove_entity(child).unwrap();
    assert_eq!(world.world_generation(), 4);
}

#[test]
fn explicit_entity_spawn_advances_generation_exactly_once() {
    let mut world = World::empty();

    assert!(world.spawn_empty_at(42).unwrap());
    assert_eq!(world.world_generation(), 1);

    assert!(!world.spawn_empty_at(42).unwrap());
    assert_eq!(world.world_generation(), 1);

    assert_eq!(
        world.spawn_empty_at(u64::MAX).unwrap_err(),
        SceneError::EntityIdExhausted { entity: u64::MAX }
    );
    assert_eq!(world.world_generation(), 1);
}

#[test]
fn imported_node_record_spawn_advances_generation_exactly_once() {
    let mut source = World::empty();
    let source_entity = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let record = source.node_record(source_entity).unwrap();
    let mut target = World::empty();

    target.insert_node_record(record).unwrap();

    assert_eq!(target.world_generation(), 1);
}

#[test]
fn fixed_presence_rebuild_marks_derived_state_before_lifecycle_callbacks() {
    let mut world = World::empty();
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());

    let lifecycle_visibility_revision = world.lifecycle_visibility_revision();
    let observations = Arc::new(Mutex::new(Vec::new()));
    for kind in [LifecycleEventKind::Add, LifecycleEventKind::Insert] {
        let observations = observations.clone();
        world.observe_component_lifecycle::<Name>(kind, move |world, event| {
            observations.lock().unwrap().push((
                event.kind(),
                world.has_pending_scene_systems(),
                world.world_generation(),
                world.lifecycle_visibility_revision(),
                world.get::<Name>(event.entity()).is_some(),
            ));
        });
    }

    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");

    assert_eq!(world.world_generation(), 1);
    assert_eq!(
        *observations.lock().unwrap(),
        vec![
            (
                LifecycleEventKind::Add,
                true,
                0,
                lifecycle_visibility_revision + 1,
                true,
            ),
            (
                LifecycleEventKind::Insert,
                true,
                0,
                lifecycle_visibility_revision + 1,
                true,
            ),
        ]
    );
    assert!(world.get::<Name>(entity).is_some());
}

#[test]
fn rejected_imported_node_record_does_not_mutate_or_advance_generation() {
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_mobility(parent, Mobility::Dynamic).unwrap();
    let before = world.clone();
    let generation = world.world_generation();

    let mut source = World::empty();
    let entity = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mut record = source.node_record(entity).unwrap();
    record.id = parent + 1;
    record.parent = Some(parent);
    record.mobility = Mobility::Static;

    assert!(world.insert_node_record(record.clone()).is_err());
    assert_eq!(world, before);
    assert!(!world.contains_entity(record.id));
    assert_eq!(world.world_generation(), generation);
}

#[test]
fn exhausted_imported_entity_id_is_rejected_before_world_mutation() {
    let mut source = World::empty();
    let source_entity = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mut record = source.node_record(source_entity).unwrap();
    record.id = u64::MAX;

    let mut target = World::empty();
    let before = target.clone();
    let generation = target.world_generation();

    assert_eq!(
        target.insert_node_record(record),
        Err(SceneError::EntityIdExhausted { entity: u64::MAX })
    );
    assert_eq!(target, before);
    assert_eq!(target.world_generation(), generation);
}

#[test]
fn rejected_imported_node_record_batch_is_atomic() {
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_mobility(parent, Mobility::Dynamic).unwrap();
    let before = world.clone();
    let generation = world.world_generation();

    let mut source = World::empty();
    let first = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let second = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mut first_record = source.node_record(first).unwrap();
    first_record.id = parent + 1;
    let mut invalid_record = source.node_record(second).unwrap();
    invalid_record.id = parent + 2;
    invalid_record.parent = Some(parent);
    invalid_record.mobility = Mobility::Static;

    assert!(
        world
            .insert_node_records(&[first_record, invalid_record])
            .is_err()
    );
    assert_eq!(world, before);
    assert_eq!(world.world_generation(), generation);
}

#[test]
fn component_replacement_advances_the_query_generation_revision() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let before_rename = world.world_generation();

    assert!(world.rename_node(entity, "Renamed").unwrap());
    assert_eq!(world.world_generation(), before_rename + 1);

    let after_rename = world.world_generation();
    assert!(!world.rename_node(entity, "Renamed").unwrap());
    assert_eq!(world.world_generation(), after_rename);
}

#[test]
fn failed_mutable_component_lookup_does_not_advance_generation() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let generation = world.world_generation();

    assert!(world.get_mut::<Name>(entity + 1).is_none());
    assert_eq!(world.world_generation(), generation);
}

#[test]
fn world_generation_is_runtime_state_not_persisted_scene_data() {
    let mut world = World::empty();
    world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    assert_eq!(world.world_generation(), 1);

    let encoded = serde_json::to_value(&world).unwrap();
    assert!(encoded.get("world_generation").is_none());

    let restored: World = serde_json::from_value(encoded).unwrap();
    assert_eq!(restored.world_generation(), 0);
}

#[test]
fn lifecycle_visibility_revision_is_runtime_state_rebuilt_once_per_world_reconstruction() {
    let mut world = World::empty();
    world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let source_revision = world.lifecycle_visibility_revision();

    let cloned = world.clone();
    assert_eq!(
        cloned.lifecycle_visibility_revision(),
        source_revision + 1,
        "clone rebuilds component rows once after copying runtime state"
    );

    let encoded = serde_json::to_value(&world).unwrap();
    assert!(encoded.get("lifecycle_visibility_revision").is_none());

    let restored: World = serde_json::from_value(encoded).unwrap();
    assert_eq!(
        restored.lifecycle_visibility_revision(),
        1,
        "deserialize rebuilds component rows once from a fresh runtime revision"
    );
}
