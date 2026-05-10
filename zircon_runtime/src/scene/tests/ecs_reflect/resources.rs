use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectObjectAddress,
    ReflectReadRequest, ReflectSchemaRequest, ReflectSerializationStrategy, ReflectTypeInfo,
    ReflectTypePath, ReflectTypeRegistration, ReflectWriteRequest, ReflectedValue,
};

use crate::scene::ecs::Resource;
use crate::scene::{NodeKind, ReflectResource, RuntimeTypeRegistration, World};

const FRAME_COUNTER_TYPE_PATH: &str = "zircon_runtime::scene::tests::ecs_reflect::FrameCounter";

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter {
    value: u32,
}

impl Resource for FrameCounter {}

#[test]
fn manual_resource_registration_adds_reflected_resource_schema() {
    let mut world = World::empty();

    register_frame_counter_resource(&mut world);

    let registration = world
        .reflect_schema(FRAME_COUNTER_TYPE_PATH)
        .expect("resource schema should be registered");
    assert_eq!(registration.type_path.type_path, FRAME_COUNTER_TYPE_PATH);
    assert_eq!(registration.type_path.short_type_path, "FrameCounter");
    assert_eq!(registration.display_name, "Frame Counter");
    assert!(!registration.is_component);
    assert!(registration.is_resource);
    assert!(matches!(
        registration.serialization,
        ReflectSerializationStrategy::ResourceHandle
    ));
    assert_eq!(registration.type_info.fields.len(), 1);
    assert_eq!(registration.type_info.fields[0].name, "value");
    assert_eq!(registration.type_info.fields[0].value_type_path, "Unsigned");
    assert!(registration.type_info.fields[0].editable);

    let listed = world
        .list_reflect_types(ReflectSchemaRequest::for_type("FrameCounter"))
        .expect("short resource type path should resolve")
        .registrations;

    assert_eq!(listed, vec![registration]);
}

#[test]
fn resource_reflection_reads_and_writes_field_through_facade() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);
    world.insert_resource(FrameCounter { value: 7 });
    let address = frame_counter_address();

    let read = world
        .reflect_read(ReflectReadRequest::new(address.clone(), "value"))
        .expect("resource field should read through reflection");
    assert_eq!(
        read.field,
        ReflectFieldValue::new("value", ReflectedValue::Unsigned(7))
    );
    let fields = world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(address.clone()),
        )
        .expect("resource fields should enumerate through reflection")
        .fields;
    assert_eq!(
        fields,
        vec![ReflectFieldValue::new("value", ReflectedValue::Unsigned(7))]
    );

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            address.clone(),
            "value",
            ReflectedValue::Unsigned(11),
        ))
        .expect("resource field should write through reflection");

    assert!(response.changed);
    assert_eq!(
        response.field,
        ReflectFieldValue::new("value", ReflectedValue::Unsigned(11))
    );
    assert_eq!(world.get_resource::<FrameCounter>().unwrap().value, 11);

    let unchanged = world
        .reflect_write(ReflectWriteRequest::new(
            address,
            "value",
            ReflectedValue::Unsigned(11),
        ))
        .expect("same resource value should be accepted as unchanged");
    assert!(!unchanged.changed);
}

#[test]
fn resource_reflection_write_updates_change_tick() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);
    world.insert_resource(FrameCounter { value: 1 });
    let before = world
        .resource_change_ticks::<FrameCounter>()
        .expect("inserted resource should have ticks")
        .changed();

    world
        .reflect_write(ReflectWriteRequest::new(
            frame_counter_address(),
            "value",
            ReflectedValue::Unsigned(2),
        ))
        .expect("resource write should route through mutable resource access");

    let after = world
        .resource_change_ticks::<FrameCounter>()
        .expect("written resource should still have ticks")
        .changed();
    assert!(after > before);
}

#[test]
fn missing_reflected_resource_returns_structured_error() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(frame_counter_address(), "value"))
            .expect_err("missing reflected resources should be structured"),
        ReflectError::MissingResource {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_write(ReflectWriteRequest::new(
                frame_counter_address(),
                "value",
                ReflectedValue::Unsigned(1),
            ))
            .expect_err("missing reflected resource writes should be structured"),
        ReflectError::MissingResource {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        }
    );
}

#[test]
fn resource_registration_without_adapter_returns_structured_error() {
    let mut world = World::empty();
    world.type_registry_mut_for_tests().clear();
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration::metadata(
            frame_counter_registration(),
        ))
        .expect("metadata-only resource registration should be accepted");

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(frame_counter_address(), "value"))
            .expect_err("metadata-only resource should report missing resource adapter"),
        ReflectError::NoResourceAdapter {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        }
    );
}

#[test]
fn component_and_resource_reflection_share_address_and_facade_shape() {
    let mut world = World::empty();
    register_frame_counter_resource(&mut world);
    world.insert_resource(FrameCounter { value: 3 });
    let entity = world.spawn_node(NodeKind::Mesh);
    let component_address =
        ReflectObjectAddress::component(entity, "Name").expect("component address should be valid");
    let resource_address = frame_counter_address();

    let component_read = world
        .reflect_read(ReflectReadRequest::new(component_address.clone(), "value"))
        .expect("component read should use shared facade");
    let resource_read = world
        .reflect_read(ReflectReadRequest::new(resource_address.clone(), "value"))
        .expect("resource read should use shared facade");

    assert_eq!(component_read.address, component_address.clone());
    assert_eq!(resource_read.address, resource_address.clone());
    assert_eq!(component_read.field.field_name, "value");
    assert_eq!(resource_read.field.field_name, "value");

    let schema_type_paths = world
        .list_reflect_types(ReflectSchemaRequest::editor_visible())
        .expect("component and resource schemas should share schema facade")
        .registrations
        .into_iter()
        .map(|registration| registration.type_path.type_path)
        .collect::<Vec<_>>();
    assert!(schema_type_paths.contains(&"zircon_runtime::scene::components::Name".to_string()));
    assert!(schema_type_paths.contains(&FRAME_COUNTER_TYPE_PATH.to_string()));

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                ReflectObjectAddress::component(entity, FRAME_COUNTER_TYPE_PATH)
                    .expect("component-shaped resource address should be valid DTO"),
                "value",
            ))
            .expect_err("resource registration cannot be addressed as a component"),
        ReflectError::AddressKindMismatch {
            expected: format!("component `{FRAME_COUNTER_TYPE_PATH}`"),
            actual: format!("non-component `{FRAME_COUNTER_TYPE_PATH}`"),
        }
    );
    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                ReflectObjectAddress::resource("Name")
                    .expect("resource-shaped component address should be valid DTO"),
                "value",
            ))
            .expect_err("component registration cannot be addressed as a resource"),
        ReflectError::AddressKindMismatch {
            expected: "resource `zircon_runtime::scene::components::Name`".to_string(),
            actual: "non-resource `zircon_runtime::scene::components::Name`".to_string(),
        }
    );
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

fn frame_counter_address() -> ReflectObjectAddress {
    ReflectObjectAddress::resource(FRAME_COUNTER_TYPE_PATH)
        .expect("resource address should be valid")
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
