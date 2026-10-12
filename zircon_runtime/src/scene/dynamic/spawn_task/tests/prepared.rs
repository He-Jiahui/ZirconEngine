use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::scene::{
    components::Name, ecs::LifecycleEventKind, DefaultLevelManager, DynamicScene,
    DynamicSceneError, NodeKind, Resource, World,
};

use super::PreparedDynamicSceneSpawn;

#[derive(Debug, PartialEq, Eq)]
struct UnrelatedRuntimeResource(u32);

impl Resource for UnrelatedRuntimeResource {}

#[test]
fn dynamic_scene_asset_reload_staged_spawn_rejects_changed_target() {
    let mut source = World::empty();
    source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");
    let prepared =
        PreparedDynamicSceneSpawn::new(scene.clone()).expect("captured scene should prepare");
    let mut target = World::empty();
    let expected_generation = target.world_generation();
    let staged = prepared
        .stage_into(&mut target)
        .expect("scene should stage on target snapshot");

    target
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let changed_target = target.clone();
    let actual_generation = target.world_generation();
    let error = staged
        .commit_into(&mut target)
        .expect_err("stale transaction must not replace a changed target");

    assert_eq!(
        error,
        DynamicSceneError::TargetWorldChanged {
            expected_generation,
            actual_generation,
        }
    );
    assert_eq!(target, changed_target);
}

#[test]
fn dynamic_scene_asset_reload_staged_spawn_preserves_live_runtime_resources() {
    let mut source = World::empty();
    source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let prepared = PreparedDynamicSceneSpawn::new(
        DynamicScene::from_world(&source).expect("source scene should capture"),
    )
    .expect("captured scene should prepare");
    let mut target = World::empty();
    target.insert_resource(UnrelatedRuntimeResource(41));

    prepared
        .spawn_into(&mut target)
        .expect("staged scene should commit");

    assert_eq!(
        target.get_resource::<UnrelatedRuntimeResource>(),
        Some(&UnrelatedRuntimeResource(41))
    );
}

#[test]
fn dynamic_scene_asset_reload_level_transaction_is_target_bound_and_replays_live_callbacks() {
    let mut source = World::empty();
    source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");
    let prepared =
        PreparedDynamicSceneSpawn::new(scene.clone()).expect("captured scene should prepare");
    let manager = DefaultLevelManager::default();
    let first = manager.create_level(World::empty(), Default::default());
    let second = manager.create_level(World::empty(), Default::default());
    let callbacks = Arc::new(AtomicUsize::new(0));
    let observed = callbacks.clone();
    first.with_world_mut(|world| {
        world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |_, _| {
            observed.fetch_add(1, Ordering::Relaxed);
        });
    });

    let staged = prepared
        .stage_into_level(&first, 1024 * 1024)
        .expect("level scene should stage outside commit");
    let error = staged
        .commit_into_level(&second)
        .expect_err("one level transaction must not commit into another level");

    assert!(matches!(
        error,
        DynamicSceneError::TargetLevelChanged { .. }
    ));
    assert_eq!(callbacks.load(Ordering::Relaxed), 0);
    assert!(second.with_world(|world| world.node_records().is_empty()));

    PreparedDynamicSceneSpawn::new(scene)
        .expect("captured scene should prepare again")
        .stage_into_level(&first, 1024 * 1024)
        .expect("first level should stage")
        .commit_into_level(&first)
        .expect("target-bound transaction should commit to its source level");
    assert!(callbacks.load(Ordering::Relaxed) > 0);
}
