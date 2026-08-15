use zircon_runtime_interface::reflect::{ReflectError, ReflectFieldValue, ReflectedValue};

use crate::scene::components::ActiveInHierarchy;
use crate::scene::{
    derived_component_registration_with_adapter, EntityId, ReflectComponent,
    RuntimeTypeRegistration, World,
};

use super::component_support;

const TYPE_PATH: &str = "zircon_runtime::scene::components::ActiveInHierarchy";

pub(super) fn registration() -> Result<RuntimeTypeRegistration, ReflectError> {
    derived_component_registration_with_adapter::<ActiveInHierarchy>(
        ReflectComponent::new(
            TYPE_PATH,
            contains,
            read_field,
            read_fields,
            write_field,
            remove,
        )
        .with_dense_field_slots(read_field_by_slot, write_field_by_slot)
        .with_dense_field_batch_write(write_fields_by_slot),
    )
}

fn contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<ActiveInHierarchy>(entity).is_some() || world.contains_entity(entity)
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    if field_name != "value" {
        return Err(component_support::unknown(TYPE_PATH, field_name));
    }
    read_value(world, entity)
}

fn read_value(world: &World, entity: EntityId) -> Result<ReflectedValue, ReflectError> {
    let Some(value) = world.active_in_hierarchy(entity) else {
        return Err(component_support::missing(entity, TYPE_PATH));
    };
    Ok(ReflectedValue::Bool(value))
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "value",
        read_value(world, entity)?,
    )])
}

fn write_field(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    field_name: &str,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if field_name != "value" {
        return Err(component_support::unknown(TYPE_PATH, field_name));
    }
    write_value()
}

fn write_value() -> Result<bool, ReflectError> {
    Err(ReflectError::NonEditableField {
        type_path: TYPE_PATH.to_string(),
        field_name: "value".to_string(),
    })
}

fn read_field_by_slot(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    match field_slot {
        0 => read_value(world, entity),
        _ => Err(component_support::unknown(
            TYPE_PATH,
            &format!("#{field_slot}"),
        )),
    }
}

fn write_field_by_slot(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    field_slot: u32,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    match field_slot {
        0 => write_value(),
        _ => Err(component_support::unknown(
            TYPE_PATH,
            &format!("#{field_slot}"),
        )),
    }
}

fn write_fields_by_slot(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    if fields.is_empty() {
        return Ok(false);
    }
    for (field_slot, _) in fields {
        if field_slot != 0 {
            return Err(component_support::unknown(
                TYPE_PATH,
                &format!("#{field_slot}"),
            ));
        }
    }
    write_value()
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    component_support::remove::<ActiveInHierarchy>(world, entity, TYPE_PATH)
}
