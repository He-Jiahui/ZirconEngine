use serde_json::{json, Value};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue,
    ReflectScriptVisibility, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
    ReflectTypeRegistration, ReflectedValue,
};

use crate::scene::{DynamicScene, DynamicSceneError, EntityId, NodeKind, SceneError, World};

use super::VmTypeBacking;

const TYPE_PATH: &str = "probe.Component.Scratch";

#[test]
fn reflect163_versioned_scene_restores_vm_transient_default() {
    let schema = scratch_registration(Some(42));
    let raw = json!({ "saved": 7, "scratch": 99 });
    let (source, source_id) = vm_fixture(schema.clone(), raw.clone());
    assert_payload(&source, source_id, &raw);

    let scene = versioned_round_trip(&capture(&source));
    let entity = &scene.entities[0];
    assert_eq!(entity.source_entity, source_id);
    let component = entity
        .components
        .iter()
        .find(|c| c.type_path == TYPE_PATH)
        .unwrap();
    assert!(component.plugin_owned);
    assert_eq!(component.fields.len(), 1);
    let saved = &component.fields[0];
    assert_eq!(saved.field_id, field_id("saved"));
    assert_eq!(saved.value, ReflectedValue::Integer(7));
    let descriptor = &scene.component_types[0];
    assert_eq!(descriptor.type_id, TYPE_PATH);
    assert!(descriptor.properties.iter().any(|f| f.name == "scratch"));

    let sentinel = json!({ "saved": -5, "scratch": 13 });
    let (mut target, existing) = vm_fixture(schema, sentinel.clone());
    let remap = scene
        .spawn_into(&mut target)
        .expect("the registered VM schema should restore the omitted scratch default");
    let restored = remap.get(source_id).expect("source entity should map");

    assert_ne!(restored, existing);
    assert_eq!(target.entity_count(), 2);
    assert_payload(&target, restored, &json!({ "saved": 7, "scratch": 42 }));
    assert_payload(&target, existing, &sentinel);
    assert_payload(&source, source_id, &raw);
}

#[test]
fn reflect163_direct_vm_write_rejects_missing_transient_field() {
    let original = json!({ "saved": 8, "scratch": 99 });
    let (mut world, existing) = vm_fixture(scratch_registration(Some(42)), original.clone());
    let error = world
        .set_dynamic_component(existing, TYPE_PATH, json!({ "saved": 7 }))
        .expect_err("a direct VM write must still provide the full object");
    assert_eq!(error, missing_field("scratch"));
    assert_payload(&world, existing, &original);
}

#[test]
fn reflect163_scene_rejects_missing_persistent_field_with_default() {
    // Both fields persist in this control: only the second row is missing `saved`.
    let schema = vm_registration(vec![
        saved_field(),
        ReflectFieldInfo::new(
            field_id("enabled"),
            "enabled",
            "Bool",
            ReflectEditorHint::Bool,
        ),
    ]);
    let (mut source, _) = vm_fixture(schema.clone(), json!({ "saved": 5, "enabled": false }));
    let incomplete = attach(&mut source, json!({ "saved": 7, "enabled": true }));
    let mut fixture = capture(&source);
    instance_fields(&mut fixture, incomplete).retain(|field| field.field_id != field_id("saved"));
    let scene = versioned_round_trip(&fixture);

    let sentinel = json!({ "saved": -5, "enabled": true });
    let (mut target, existing) = vm_fixture(schema, sentinel.clone());
    assert_spawn_rejected(&scene, &mut target, "saved");
    assert_payload(&target, existing, &sentinel);
}

#[test]
fn reflect163_scene_rejects_missing_transient_field_without_default() {
    let schema = scratch_registration(None);
    let (source, _) = vm_fixture(schema.clone(), json!({ "saved": 7, "scratch": 99 }));
    let scene = versioned_round_trip(&capture(&source));
    let sentinel = json!({ "saved": -5, "scratch": 13 });
    let (mut target, existing) = vm_fixture(schema, sentinel.clone());
    assert_spawn_rejected(&scene, &mut target, "scratch");
    assert_payload(&target, existing, &sentinel);
}

#[test]
fn reflect163_descriptor_only_restore_keeps_snapshot_sparse() {
    let raw = json!({ "saved": 7, "scratch": 99 });
    let (source, source_id) = vm_fixture(scratch_registration(Some(42)), raw);
    let scene = versioned_round_trip(&capture(&source));

    // No VM registration on the target: spawn imports the archived descriptor.
    let mut target = World::empty();
    let remap = scene
        .spawn_into(&mut target)
        .expect("descriptor-only spawn");
    let restored = remap.get(source_id).expect("source entity should map");
    assert!(target.component_type_descriptor(TYPE_PATH).is_some());
    assert_payload(&target, restored, &json!({ "saved": 7 }));
}

#[cfg(feature = "native-host")]
#[test]
fn reflect163_prepared_spawn_budgets_reconstructed_defaults() {
    use crate::scene::PreparedDynamicSceneSpawn;

    const HEADROOM: usize = 1024 * 1024;
    let schema = scratch_registration(Some(42));
    let (mut source, first) = vm_fixture(schema.clone(), json!({ "saved": 7, "scratch": 99 }));
    let second = attach(&mut source, json!({ "saved": 8, "scratch": 100 }));
    let sparse = capture(&source);
    let scratch =
        ReflectFieldValue::new(field_id("scratch"), "scratch", ReflectedValue::Integer(91));
    let mut partial = sparse.clone();
    instance_fields(&mut partial, first).push(scratch.clone());
    let mut explicit = partial.clone();
    instance_fields(&mut explicit, second).push(scratch);
    let prepare = |scene| PreparedDynamicSceneSpawn::new(versioned_round_trip(scene)).unwrap();
    let explicit = prepare(&explicit);
    let partial = prepare(&partial);
    let sparse = prepare(&sparse);

    let sentinel = json!({ "saved": -5, "scratch": 13 });
    let (mut target, existing) = vm_fixture(schema, sentinel.clone());
    let mut estimate = |scene: &PreparedDynamicSceneSpawn, limit| {
        scene
            .capture_world_target(&mut target, limit)
            .map(|snapshot| snapshot.estimated_bytes())
    };
    // Stored values are paid for by the document; a default costs each missing instance.
    let explicit_bytes = estimate(&explicit, HEADROOM).expect("estimate zero defaults");
    let limit = estimate(&partial, HEADROOM).expect("estimate one default");
    let sparse_bytes = estimate(&sparse, HEADROOM).expect("estimate two defaults");
    assert!(
        explicit_bytes < limit && limit < sparse_bytes,
        "expected strictly increasing target bytes for 0/1/2 defaults, got {explicit_bytes}/{limit}/{sparse_bytes}"
    );
    assert_eq!(estimate(&partial, limit).unwrap(), limit);
    let bounded = estimate(&sparse, limit);
    assert!(
        matches!(
            &bounded,
            Err(DynamicSceneError::TargetSnapshotTooLarge { estimated_bytes, limit_bytes })
                if *estimated_bytes > limit && *limit_bytes == limit
        ),
        "expected a target budget rejection at {limit} bytes, got {bounded:?}"
    );
    assert_eq!(target.entity_count(), 1);
    assert_payload(&target, existing, &sentinel);
}

fn capture(source: &World) -> DynamicScene {
    DynamicScene::from_world(source).expect("source scene should capture")
}

fn versioned_round_trip(scene: &DynamicScene) -> DynamicScene {
    let text = scene.to_versioned_json_pretty().unwrap();
    DynamicScene::from_versioned_json(&text).expect("versioned scene should decode")
}

fn instance_fields(scene: &mut DynamicScene, id: EntityId) -> &mut Vec<ReflectFieldValue> {
    &mut scene
        .entities
        .iter_mut()
        .filter(|entity| entity.source_entity == id)
        .flat_map(|entity| &mut entity.components)
        .find(|component| component.type_path == TYPE_PATH)
        .expect("snapshot should contain the fixture component")
        .fields
}

fn vm_fixture(schema: ReflectTypeRegistration, value: Value) -> (World, EntityId) {
    let mut world = World::empty();
    world
        .register_vm_type(schema, VmTypeBacking::DynamicComponent)
        .expect("public plugin VM schema should register");
    let entity = attach(&mut world, value);
    (world, entity)
}

fn attach(world: &mut World, value: Value) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_dynamic_component(entity, TYPE_PATH, value)
        .expect("attach full payload");
    entity
}

#[track_caller]
fn assert_payload(world: &World, entity: EntityId, expected: &Value) {
    assert_eq!(world.dynamic_component(entity, TYPE_PATH), Some(expected));
}

fn scratch_registration(default: Option<i64>) -> ReflectTypeRegistration {
    let mut scratch = integer_field("scratch")
        .with_serializable(false)
        .with_editor_visible(false)
        .with_editable(false);
    scratch.default_value = default.map(ReflectedValue::Integer);
    vm_registration(vec![saved_field(), scratch])
}

fn saved_field() -> ReflectFieldInfo {
    integer_field("saved").with_default_value(ReflectedValue::Integer(17))
}

fn integer_field(name: &str) -> ReflectFieldInfo {
    ReflectFieldInfo::new(field_id(name), name, "Integer", ReflectEditorHint::Integer)
}

fn field_id(name: &str) -> ReflectFieldId {
    // Use the same stable keys as the plugin descriptor's ordinary projection.
    ReflectFieldId::from_stable_keys(TYPE_PATH, name)
}

fn vm_registration(fields: Vec<ReflectFieldInfo>) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(TYPE_PATH, "Scratch").unwrap(),
        "Scratch",
        ReflectTypeInfo::json_with_fields(fields),
        ReflectSerializationStrategy::Json,
    )
    .as_component()
    .with_plugin_id("probe")
    .unwrap()
    .with_script_visibility(ReflectScriptVisibility::Public)
}

fn missing_field(field: &str) -> SceneError {
    SceneError::Reflect(ReflectError::UnknownField {
        type_path: TYPE_PATH.to_owned(),
        field_name: field.to_owned(),
    })
}

#[track_caller]
fn assert_spawn_rejected(scene: &DynamicScene, target: &mut World, field: &str) {
    match scene
        .spawn_into(target)
        .expect_err("incomplete VM payload should fail")
    {
        DynamicSceneError::WorldMutation(cause) => assert_eq!(cause, missing_field(field)),
        other => panic!("expected missing {TYPE_PATH}.{field} from World writer, got {other:?}"),
    }
    assert_eq!(target.entity_count(), 1);
}
