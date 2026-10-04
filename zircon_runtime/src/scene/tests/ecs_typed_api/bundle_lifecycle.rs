//! Bundle 预检失败不得公布部分实体或组件；生命周期观察者只能看到提交后的最终原型签名。

use super::*;

#[test]
fn bundle_preflight_rejects_a_later_protected_component_without_publishing_earlier_components() {
    let mut world = World::new();
    let entity = world
        .spawn_node(crate::scene::NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let original_transform = world
        .get::<LocalTransform>(entity)
        .expect("mesh nodes must have a local transform")
        .transform;
    let generation_before = world.world_generation();
    assert!(matches!(
        world.insert_bundle(
            entity,
            (
                Health(42),
                LocalTransform {
                    transform: original_transform,
                },
            ),
        ),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "LocalTransform",
            operation: "insert",
        })
    ));

    assert!(!world.contains_component::<Health>(entity));
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(
        world
            .get::<LocalTransform>(entity)
            .map(|component| component.transform),
        Some(original_transform)
    );
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn protected_bundle_spawn_preflight_does_not_publish_a_default_entity() {
    let mut world = World::empty();
    let generation_before = world.world_generation();
    assert!(matches!(
        world.spawn((
            Health(42),
            LocalTransform {
                transform: Transform::default(),
            },
        )),
        Err(SceneError::ProtectedAuthoredComponentMutation {
            component: "LocalTransform",
            operation: "insert",
        })
    ));

    assert!(world.node_record(1).is_none());
    assert!(!world.contains_component::<Health>(1));
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn node_record_transform_preflight_rejects_zero_scale_without_publication() {
    let mut world = World::empty();
    let entity = world.spawn(()).expect("fixture node must spawn");
    let mut record = world.node_record(entity).expect("fixture must project");
    world
        .remove_entity(entity)
        .expect("fixture node must detach");
    let generation_before = world.world_generation();
    record.transform.scale.z = 0.0;

    assert_eq!(
        world.insert_node_record(record),
        Err(SceneError::ZeroScaleTransform { entity, axis: "z" })
    );
    assert!(!world.contains_entity(entity));
    assert!(world.node_record(entity).is_none());
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn bundle_preflight_rejects_duplicate_component_types_without_publishing() {
    let mut world = World::empty();
    let generation_before = world.world_generation();

    assert!(matches!(
        world.spawn((Health(7), Health(9))),
        Err(SceneError::DuplicateBundleComponentType)
    ));

    assert!(world.node_record(1).is_none());
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn bundle_spawn_preflights_default_node_and_custom_component_ids_together() {
    let mut world = World::empty();

    let entity = world
        .spawn((Health(7),))
        .expect("bundle spawn must assign custom ids after default node components");

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert!(world.node_record(entity).is_some());
}

#[test]
fn spawn_node_lifecycle_observers_see_the_final_fixed_component_signature() {
    use std::sync::{Arc, Mutex};

    let mut world = World::empty();
    let local_transform_id = world.component_id::<LocalTransform>();
    let saw_final_signature = Arc::new(Mutex::new(false));
    let saw_final_signature_from_observer = Arc::clone(&saw_final_signature);
    world.observe_component_lifecycle::<Name>(
        crate::scene::ecs::LifecycleEventKind::Add,
        move |world, event| {
            *saw_final_signature_from_observer
                .lock()
                .expect("observer state lock") = world
                .node_record(event.entity())
                .is_some_and(|_| world.contains_component_id(event.entity(), local_transform_id));
        },
    );

    world
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");

    assert!(*saw_final_signature.lock().expect("observer state lock"));
}

#[test]
fn bundle_lifecycle_observers_see_the_final_component_signature() {
    use std::sync::{Arc, Mutex};

    let mut world = World::empty();
    let entity = world
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mana_id = world.component_id::<Mana>();
    let saw_final_signature = Arc::new(Mutex::new(false));
    let saw_final_signature_from_observer = Arc::clone(&saw_final_signature);
    world.observe_component_lifecycle::<Health>(
        crate::scene::ecs::LifecycleEventKind::Add,
        move |world, _event| {
            *saw_final_signature_from_observer
                .lock()
                .expect("observer state lock") = world.contains_component_id(entity, mana_id);
        },
    );

    world
        .insert_bundle(entity, (Health(7), Mana(9)))
        .expect("validated bundle must commit");

    assert!(*saw_final_signature.lock().expect("observer state lock"));
}

#[test]
fn bundle_spawn_lifecycle_observers_see_the_final_component_signature() {
    use std::sync::{Arc, Mutex};

    let mut world = World::empty();
    let mana_id = world.component_id::<Mana>();
    let saw_final_signature = Arc::new(Mutex::new(false));
    let saw_final_signature_from_observer = Arc::clone(&saw_final_signature);
    world.observe_component_lifecycle::<Health>(
        crate::scene::ecs::LifecycleEventKind::Add,
        move |world, event| {
            *saw_final_signature_from_observer
                .lock()
                .expect("observer state lock") = world
                .node_record(event.entity())
                .is_some_and(|_| world.contains_component_id(event.entity(), mana_id));
        },
    );

    let entity = world
        .spawn((Health(7), Mana(9)))
        .expect("validated bundle spawn must commit");

    assert_eq!(entity, 1);
    assert!(*saw_final_signature.lock().expect("observer state lock"));
}
