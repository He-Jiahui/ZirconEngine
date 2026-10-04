use std::sync::{Arc, Mutex};

use crate::scene::{
    components::{ActiveSelf, Hierarchy, LocalTransform, Mobility, Name, RenderLayerMask},
    ecs::{Component, LifecycleEventKind, StorageType},
    NodeKind, World,
};

#[derive(Debug, PartialEq, Eq)]
struct RebuiltArchetypeMarker;

impl Component for RebuiltArchetypeMarker {}

#[derive(Debug, PartialEq, Eq)]
struct RebuiltSparseArchetypeMarker;

impl Component for RebuiltSparseArchetypeMarker {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

fn node_record_with_id(id: u64) -> crate::scene::components::NodeRecord {
    let mut source = World::empty();
    let entity = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mut record = source
        .node_record(entity)
        .expect("spawned node must produce a record");
    record.id = id;
    record
}

#[test]
fn node_record_batch_publishes_once_after_all_records_are_visible() {
    let mut world = World::empty();
    let mut first = node_record_with_id(41);
    first.name = "First".to_string();
    let mut second = node_record_with_id(42);
    second.name = "Second".to_string();
    second.parent = Some(first.id);

    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_for_callback = Arc::clone(&observed);
    let second_id = second.id;
    let name_component_id = world.component_id::<Name>();
    world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |world, event| {
        observed_for_callback
            .lock()
            .expect("test observer lock")
            .push((
                event.entity(),
                world.contains_entity(second_id),
                world
                    .entity_archetype_component_ids(second_id)
                    .contains(&name_component_id),
            ));
    });

    let generation_before = world.world_generation();
    world
        .insert_node_records(&[first.clone(), second.clone()])
        .expect("validated batch must commit");

    assert_eq!(world.world_generation(), generation_before + 1);
    assert_eq!(world.node_record(first.id), Some(first));
    assert_eq!(world.node_record(second.id), Some(second));
    let observed = observed.lock().expect("test observer lock");
    assert_eq!(observed.len(), 2);
    assert!(observed
        .iter()
        .all(|(_, second_visible, second_has_final_signature)| {
            *second_visible && *second_has_final_signature
        }));
}

#[test]
fn node_record_batch_assigns_the_complete_fixed_component_signature() {
    let mut world = World::empty();
    let record = node_record_with_id(41);

    world
        .insert_node_record(record.clone())
        .expect("validated record must commit with its final signature");

    let name = world.component_id::<Name>();
    let hierarchy = world.component_id::<Hierarchy>();
    let transform = world.component_id::<LocalTransform>();
    let active = world.component_id::<ActiveSelf>();
    let render_layer_mask = world.component_id::<RenderLayerMask>();
    let mobility = world.component_id::<Mobility>();
    let signature = world.entity_archetype_component_ids(record.id);
    assert!(signature.contains(&name));
    assert!(signature.contains(&hierarchy));
    assert!(signature.contains(&transform));
    assert!(signature.contains(&active));
    assert!(signature.contains(&render_layer_mask));
    assert!(signature.contains(&mobility));
}

#[test]
fn pending_component_row_publishes_dense_and_sparse_values_in_one_transition() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let marker = world.component_id::<RebuiltArchetypeMarker>();
    let sparse_marker = world.component_id::<RebuiltSparseArchetypeMarker>();
    let assignments_before = world.archetype_assignment_count();

    let mut row = world.begin_component_row(entity);
    world.stage_component_row_value(&mut row, RebuiltArchetypeMarker);
    world.stage_component_row_value(&mut row, RebuiltSparseArchetypeMarker);
    world.commit_component_row(entity, row, true);

    assert!(
        world
            .entity_archetype_component_ids(entity)
            .contains(&marker),
        "final signature reconstruction must not depend on a fixed component whitelist"
    );
    assert!(
        world
            .entity_archetype_component_ids(entity)
            .contains(&sparse_marker),
        "final signature reconstruction must preserve sparse canonical storage rows"
    );
    assert!(world.get::<RebuiltArchetypeMarker>(entity).is_some());
    assert!(world.get::<RebuiltSparseArchetypeMarker>(entity).is_some());
    assert_eq!(world.archetype_assignment_count() - assignments_before, 1);
}

#[test]
fn node_record_batch_publishes_one_final_archetype_assignment_per_record() {
    let mut world = World::empty();
    let first = node_record_with_id(41);
    let second = node_record_with_id(42);
    let assignments_before = world.archetype_assignment_count();

    world
        .insert_owned_node_records(vec![first, second])
        .expect("prevalidated records should publish their final signatures");

    assert_eq!(world.archetype_assignment_count() - assignments_before, 2);
}

#[test]
fn duplicate_node_record_batch_leaves_world_unchanged() {
    let mut world = World::empty();
    let record = node_record_with_id(41);
    let generation_before = world.world_generation();

    let error = world
        .insert_node_records(&[record.clone(), record])
        .expect_err("duplicate batch identities must fail before commit");

    assert!(matches!(
        error,
        crate::scene::SceneError::DuplicateEntity { .. }
    ));
    assert_eq!(world.world_generation(), generation_before);
    assert!(world.node_records().is_empty());
}

#[test]
fn stale_registry_identity_rejects_the_entire_node_record_batch_before_commit() {
    let mut world = World::empty();
    world
        .register_stable_entity(42)
        .expect("test fixture should create a registry-only identity");
    let first = node_record_with_id(41);
    let second = node_record_with_id(42);
    let generation_before = world.world_generation();

    let error = world
        .insert_owned_node_records(vec![first, second])
        .expect_err("registry identity must be rejected during batch prevalidation");

    assert!(matches!(
        error,
        crate::scene::SceneError::DuplicateEntity { entity: 42 }
    ));
    assert_eq!(world.world_generation(), generation_before);
    assert!(world.node_records().is_empty());
}

#[test]
fn dynamic_parent_with_static_incoming_child_rejects_the_whole_batch() {
    let mut world = World::empty();
    let mut parent = node_record_with_id(41);
    parent.mobility = Mobility::Dynamic;
    let mut child = node_record_with_id(42);
    child.parent = Some(parent.id);
    let generation_before = world.world_generation();

    let error = world
        .insert_owned_node_records(vec![parent, child])
        .expect_err("dynamic parents cannot gain static children in a batch");

    assert!(matches!(
        error,
        crate::scene::SceneError::DynamicMobilityWithStaticChildren { entity: 41 }
    ));
    assert_eq!(world.world_generation(), generation_before);
    assert!(world.node_records().is_empty());
}
