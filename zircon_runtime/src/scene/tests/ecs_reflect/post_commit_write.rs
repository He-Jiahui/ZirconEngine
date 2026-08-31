use std::cell::Cell;

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue,
    ReflectObjectAddress, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
    ReflectTypeRegistration, ReflectWriteRequest, ReflectedValue,
};

use crate::scene::{EntityId, NodeKind, ReflectComponent, RuntimeTypeRegistration, World};

const WRITE_ONLY_TYPE_PATH: &str = "test.Component.WriteOnly";

thread_local! {
    static WRITE_ONLY_WRITE_CALLS: Cell<usize> = const { Cell::new(0) };
}

#[test]
fn reflection_write_returns_the_published_request_when_readback_fails() {
    let (mut world, entity) = world_with_write_only_component();

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            ReflectObjectAddress::component(entity, WRITE_ONLY_TYPE_PATH)
                .expect("write-only component address should be valid"),
            write_only_field_id("value"),
            ReflectedValue::String("Published Name".to_string()),
        ))
        .expect("a published write must not depend on a separate read adapter");

    assert!(response.changed);
    assert_eq!(
        response.field,
        ReflectFieldValue::new(
            write_only_field_id("value"),
            "value",
            ReflectedValue::String("Published Name".to_string())
        )
    );
    assert_eq!(
        world
            .find_node(entity)
            .expect("published entity should remain in the world")
            .name,
        "Published Name"
    );
}

#[test]
fn reflection_write_rejects_unknown_component_fields_before_adapter_dispatch() {
    let (mut world, entity) = world_with_write_only_component();
    reset_write_only_write_calls();

    let error = world
        .reflect_write(ReflectWriteRequest::new(
            ReflectObjectAddress::component(entity, WRITE_ONLY_TYPE_PATH)
                .expect("write-only component address should be valid"),
            write_only_field_id("unknown"),
            ReflectedValue::String("must not publish".to_string()),
        ))
        .expect_err("unknown component fields must be rejected before adapter dispatch");

    assert_eq!(
        error,
        ReflectError::UnknownField {
            type_path: WRITE_ONLY_TYPE_PATH.to_string(),
            field_name: write_only_field_id("unknown").to_string(),
        }
    );
    assert_eq!(write_only_write_calls(), 0);
}

fn world_with_write_only_component() -> (World, EntityId) {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration {
            registration: write_only_registration(),
            component: Some(write_only_component_adapter()),
            resource: None,
        })
        .expect("write-only component registration should be accepted");
    (world, entity)
}

fn write_only_registration() -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(WRITE_ONLY_TYPE_PATH, "WriteOnly")
            .expect("write-only type path should be valid"),
        "Write Only",
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::from_stable_keys(
            WRITE_ONLY_TYPE_PATH,
            "value",
            "value",
            "String",
            ReflectEditorHint::String,
        )]),
        ReflectSerializationStrategy::Value,
    )
    .as_component()
}

fn write_only_component_adapter() -> ReflectComponent {
    ReflectComponent::new(
        WRITE_ONLY_TYPE_PATH,
        write_only_contains,
        write_only_read_field,
        write_only_write_field,
        write_only_remove,
    )
    .with_dense_field_slots(
        write_only_read_field_by_slot,
        write_only_write_field_by_slot,
    )
}

fn write_only_contains(world: &World, entity: EntityId, _: &str) -> bool {
    world.contains_entity(entity)
}

fn write_only_read_field(
    _: &World,
    _: EntityId,
    _: &str,
    _: &str,
) -> Result<ReflectedValue, ReflectError> {
    Err(ReflectError::UnsupportedConversion {
        source: "test adapter intentionally rejects readback".to_string(),
        target: WRITE_ONLY_TYPE_PATH.to_string(),
    })
}

fn write_only_read_field_by_slot(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    if field_slot != 0 {
        return Err(ReflectError::UnknownField {
            type_path: type_path.to_string(),
            field_name: format!("#{field_slot}"),
        });
    }
    write_only_read_field(world, entity, type_path, "value")
}

fn write_only_write_field_by_slot(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if field_slot != 0 {
        return Err(ReflectError::UnknownField {
            type_path: type_path.to_string(),
            field_name: format!("#{field_slot}"),
        });
    }
    write_only_write_field(world, entity, type_path, "value", value)
}

fn write_only_write_field(
    world: &mut World,
    entity: EntityId,
    _: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    WRITE_ONLY_WRITE_CALLS.with(|calls| calls.set(calls.get() + 1));
    if field_name != "value" {
        return Err(ReflectError::UnknownField {
            type_path: WRITE_ONLY_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
        });
    }
    let ReflectedValue::String(value) = value else {
        return Err(ReflectError::TypeMismatch {
            type_path: WRITE_ONLY_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "String".to_string(),
            actual: "non-string".to_string(),
        });
    };
    world
        .rename_node(entity, value)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("{WRITE_ONLY_TYPE_PATH}.{field_name}"),
        })
}

fn write_only_remove(world: &mut World, entity: EntityId, _: &str) -> Result<bool, ReflectError> {
    Ok(world.contains_entity(entity))
}

fn reset_write_only_write_calls() {
    WRITE_ONLY_WRITE_CALLS.with(|calls| calls.set(0));
}

fn write_only_write_calls() -> usize {
    WRITE_ONLY_WRITE_CALLS.with(Cell::get)
}

fn write_only_field_id(field_key: &str) -> ReflectFieldId {
    ReflectFieldId::from_stable_keys(WRITE_ONLY_TYPE_PATH, field_key)
}
