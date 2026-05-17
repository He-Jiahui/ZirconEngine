use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectObjectAddress,
    ReflectReadRequest, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
    ReflectTypeRegistration, ReflectedValue,
};

use crate::plugin::ComponentTypeDescriptor;
use crate::scene::ecs::Resource;
use crate::scene::{DynamicScene, NodeKind, ReflectResource, ScenePatch, World};

const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";
const FRAME_COUNTER_TYPE_PATH: &str = "zircon_runtime::scene::tests::dynamic_scene::FrameCounter";

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter {
    value: u32,
}

impl Resource for FrameCounter {}

#[test]
fn dynamic_scene_roundtrips_reflected_components_with_entity_remap() {
    let mut source = World::empty();
    source
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic descriptor should register");
    let parent = source.spawn_node(NodeKind::Mesh);
    let child = source.spawn_node(NodeKind::Mesh);
    source
        .rename_node(parent, "Weather Root")
        .expect("parent should be named");
    source
        .rename_node(child, "Cloud")
        .expect("child should be named");
    source
        .set_parent_checked(child, Some(parent))
        .expect("child should be parented");
    source
        .set_dynamic_component(
            child,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.75, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    let encoded = serde_json::to_string(
        &DynamicScene::from_world(&source).expect("source world should export"),
    )
    .expect("dynamic scene should serialize");
    assert!(encoded.contains("\"format_version\":1"));
    assert!(encoded.contains(CLOUD_LAYER_TYPE_PATH));
    let scene: DynamicScene =
        serde_json::from_str(&encoded).expect("dynamic scene should deserialize");

    let mut target = World::empty();
    target
        .register_component_type(cloud_layer_descriptor())
        .expect("target descriptor should register");
    let collision = target.spawn_node(NodeKind::Mesh);
    assert_eq!(collision, parent);

    let remap = scene
        .spawn_into(&mut target)
        .expect("scene should spawn into target world");
    let mapped_parent = remap
        .get(parent)
        .expect("parent should have a target mapping");
    let mapped_child = remap
        .get(child)
        .expect("child should have a target mapping");

    assert_ne!(mapped_parent, parent);
    assert_eq!(target.parent_of(mapped_child), Some(mapped_parent));
    assert_eq!(
        target
            .find_node(mapped_child)
            .expect("mapped child should exist")
            .name,
        "Cloud"
    );
    assert_eq!(
        target.dynamic_component(mapped_child, CLOUD_LAYER_TYPE_PATH),
        Some(&json!({ "coverage": 0.75, "label": "storm front" }))
    );
    assert_eq!(
        target
            .reflect_read(ReflectReadRequest::new(
                ReflectObjectAddress::component(mapped_child, CLOUD_LAYER_TYPE_PATH)
                    .expect("component address should be valid"),
                "coverage",
            ))
            .expect("spawned dynamic field should read through reflection")
            .field,
        ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.75))
    );
}

#[test]
fn scene_patch_applies_reflected_resources() {
    let mut source = World::empty();
    register_frame_counter_resource(&mut source);
    source.insert_resource(FrameCounter { value: 7 });

    let patch = ScenePatch::from_scene(
        DynamicScene::from_world(&source).expect("resource world should export"),
    );

    let mut target = World::empty();
    register_frame_counter_resource(&mut target);
    target.insert_resource(FrameCounter { value: 0 });

    let remap = patch
        .apply(&mut target)
        .expect("resource patch should apply");

    assert!(remap.is_empty());
    assert_eq!(
        target
            .get_resource::<FrameCounter>()
            .expect("target resource should still exist")
            .value,
        7
    );
}

#[test]
fn versioned_json_migrates_legacy_world_project_documents() {
    let mut legacy = World::empty();
    let entity = legacy.spawn_node(NodeKind::Mesh);
    legacy
        .rename_node(entity, "Legacy Mesh")
        .expect("legacy entity should be named");
    let legacy_json = serde_json::to_string(&json!({
        "format_version": 2,
        "world": legacy,
    }))
    .expect("legacy project document should serialize");

    let scene =
        DynamicScene::from_versioned_json(&legacy_json).expect("legacy world should migrate");
    let migrated = scene
        .entities
        .iter()
        .find(|entity| entity.source_entity == 1)
        .expect("legacy entity should be migrated");

    assert_eq!(migrated.record.name, "Legacy Mesh");
    assert_eq!(migrated.record.kind, NodeKind::Mesh);

    let encoded = scene
        .to_versioned_json_pretty()
        .expect("dynamic scene should write versioned JSON");
    let decoded =
        DynamicScene::from_versioned_json(&encoded).expect("dynamic scene JSON should reload");
    assert_eq!(decoded, scene);
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}

fn register_frame_counter_resource(world: &mut World) {
    world
        .type_registry_mut_for_tests()
        .register_resource(frame_counter_registration(), frame_counter_adapter())
        .expect("frame counter resource registration should be accepted");
}

fn frame_counter_registration() -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(FRAME_COUNTER_TYPE_PATH, "FrameCounter")
            .expect("frame counter type path should be valid"),
        "Frame Counter",
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
            "value",
            "Unsigned",
            ReflectEditorHint::Unsigned,
        )]),
        ReflectSerializationStrategy::ResourceHandle,
    )
    .as_resource()
    .with_remote_visible(true)
}

fn frame_counter_adapter() -> ReflectResource {
    ReflectResource {
        contains: frame_counter_contains,
        read_field: frame_counter_read_field,
        read_fields: frame_counter_read_fields,
        write_field: frame_counter_write_field,
    }
}

fn frame_counter_contains(world: &World) -> bool {
    world.get_resource::<FrameCounter>().is_some()
}

fn frame_counter_read_field(
    world: &World,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let resource = world
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    match field_name {
        "value" => Ok(ReflectedValue::Unsigned(resource.value as u64)),
        _ => Err(unknown_frame_counter_field(field_name)),
    }
}

fn frame_counter_read_fields(world: &World) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "value",
        frame_counter_read_field(world, "value")?,
    )])
}

fn frame_counter_write_field(
    world: &mut World,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let current = world
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    if field_name != "value" {
        return Err(unknown_frame_counter_field(field_name));
    }
    let next = expect_frame_counter_value(field_name, value)?;
    if current.value == next {
        return Ok(false);
    }

    world
        .get_resource_mut::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?
        .value = next;
    Ok(true)
}

fn expect_frame_counter_value(
    field_name: &str,
    value: ReflectedValue,
) -> Result<u32, ReflectError> {
    match value {
        ReflectedValue::Unsigned(value) if u32::try_from(value).is_ok() => Ok(value as u32),
        ReflectedValue::Unsigned(_) => Err(ReflectError::TypeMismatch {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "u32 Unsigned".to_string(),
            actual: "Unsigned".to_string(),
        }),
        value => Err(ReflectError::TypeMismatch {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "Unsigned".to_string(),
            actual: value.type_name().to_string(),
        }),
    }
}

fn missing_frame_counter_resource() -> ReflectError {
    ReflectError::MissingResource {
        type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
    }
}

fn unknown_frame_counter_field(field_name: &str) -> ReflectError {
    ReflectError::UnknownField {
        type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        field_name: field_name.to_string(),
    }
}
