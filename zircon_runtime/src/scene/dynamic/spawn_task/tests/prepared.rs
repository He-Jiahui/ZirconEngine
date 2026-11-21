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
        world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            // SAFETY: Only an external atomic counter changes; retained World payload and
            // dynamic JSON are untouched.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                    observed.fetch_add(1, Ordering::Relaxed);
                })
            },
        );
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

#[test]
fn prepared_vm_default_estimate_is_limit_independent() {
    use serde_json::json;
    use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue,
        ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
        ReflectedValue,
    };

    use crate::scene::VmTypeBacking;

    const TYPE_PATH: &str = "prepared.Component.Scratch";
    const HEADROOM: usize = 1024 * 1024;
    let registered_world = |field| {
        let schema = ReflectTypeRegistration::new(
            ReflectTypePath::new(TYPE_PATH, "Scratch").unwrap(),
            "Scratch",
            ReflectTypeInfo::json_with_fields(vec![field]),
            ReflectSerializationStrategy::Json,
        )
        .as_component()
        .with_plugin_id("prepared")
        .unwrap();
        let mut world = World::empty();
        world
            .register_vm_type(schema, VmTypeBacking::DynamicComponent)
            .unwrap();
        world
    };
    let field_id = ReflectFieldId::from_stable_keys(TYPE_PATH, "scratch");
    let field = ReflectFieldInfo::new(field_id, "scratch", "String", ReflectEditorHint::String)
        .with_serializable(false)
        .with_editor_visible(false);
    let defaulted = field
        .clone()
        .with_default_value(ReflectedValue::String("target".into()));
    let mut source = registered_world(defaulted.clone());
    let id = source.spawn_node(NodeKind::Empty).unwrap();
    source
        .set_dynamic_component(id, TYPE_PATH, json!({ "scratch": "source" }))
        .unwrap();
    let mut scene = DynamicScene::from_world(&source).unwrap();
    let sparse = PreparedDynamicSceneSpawn::new(scene.clone()).unwrap();
    let component = scene.entities[0]
        .components
        .iter_mut()
        .find(|c| c.type_path == TYPE_PATH)
        .unwrap();
    assert!(component.fields.is_empty());
    component.fields.push(ReflectFieldValue::new(
        field_id,
        "scratch",
        ReflectedValue::String("stored".into()),
    ));
    let explicit = PreparedDynamicSceneSpawn::new(scene).unwrap();

    let mut target = registered_world(defaulted);
    let mut estimate = |limit| {
        sparse
            .capture_world_target(&mut target, limit)
            .unwrap()
            .estimated_bytes()
    };
    let unlimited = estimate(usize::MAX);
    assert_eq!(
        unlimited,
        estimate(HEADROOM),
        "the limit must not change the estimate"
    );

    // With everything stored, a declared default must add no target snapshot cost.
    let explicit_limit = explicit
        .capture_world_target(&mut registered_world(field), HEADROOM)
        .unwrap()
        .estimated_bytes();
    assert!(
        unlimited > explicit_limit,
        "reconstruction must have a target cost"
    );
    let snapshot = explicit
        .capture_world_target(&mut target, explicit_limit)
        .expect("stored field must fit the no-default budget");
    assert_eq!(snapshot.estimated_bytes(), explicit_limit);
    let remap = explicit
        .stage_target(snapshot)
        .unwrap()
        .commit_into(&mut target)
        .unwrap();
    let restored = remap.get(id).unwrap();
    assert_eq!(
        target.dynamic_component(restored, TYPE_PATH),
        Some(&json!({ "scratch": "stored" }))
    );
}
