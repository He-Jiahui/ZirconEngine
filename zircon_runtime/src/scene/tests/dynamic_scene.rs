use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectObjectAddress,
    ReflectReadRequest, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
    ReflectTypeRegistration, ReflectedValue,
};

use crate::core::framework::physics::PhysicsWorldStepPlan;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::ecs::Resource;
use crate::scene::{
    DefaultLevelManager, DynamicScene, DynamicSceneError, NodeKind, PreparedDynamicSceneSpawn,
    ReflectResource, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
    RuntimeSessionSlot, SceneError, ScenePatch, World,
};

use super::authoring_boundary::{
    SERIALIZED_AUTHORING_TOKENS, assert_text_excludes_authoring_tokens,
};

const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";
const FRAME_COUNTER_TYPE_PATH: &str = "zircon_runtime::scene::tests::dynamic_scene::FrameCounter";

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter {
    value: u32,
}

impl Resource for FrameCounter {}

mod archive_core;
mod archive_manifest;
mod archive_mutation;
mod level_apply;
mod scene_patch_document;

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

fn register_frame_counter_resource_with_ensure(world: &mut World) {
    world
        .type_registry_mut_for_tests()
        .register_resource(
            frame_counter_registration(),
            frame_counter_adapter_with_ensure(),
        )
        .expect("ensure-backed frame counter resource registration should be accepted");
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
        estimate_stage_clone_bytes: Some(frame_counter_stage_clone_bytes),
        stage_clone: Some(frame_counter_stage_clone),
        ensure: None,
        contains: frame_counter_contains,
        read_field: frame_counter_read_field,
        read_fields: frame_counter_read_fields,
        write_field: frame_counter_write_field,
    }
}

fn frame_counter_stage_clone_bytes(source: &World) -> Result<usize, ReflectError> {
    source
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    Ok(std::mem::size_of::<FrameCounter>())
}

fn frame_counter_stage_clone(source: &World, target: &mut World) -> Result<(), ReflectError> {
    let source = source
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    target.insert_resource(FrameCounter {
        value: source.value,
    });
    Ok(())
}

fn frame_counter_adapter_with_ensure() -> ReflectResource {
    ReflectResource {
        ensure: Some(frame_counter_ensure),
        ..frame_counter_adapter()
    }
}

fn frame_counter_ensure(world: &mut World) -> Result<bool, ReflectError> {
    if world.get_resource::<FrameCounter>().is_some() {
        return Ok(false);
    }
    world.insert_resource(FrameCounter { value: 0 });
    Ok(true)
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
