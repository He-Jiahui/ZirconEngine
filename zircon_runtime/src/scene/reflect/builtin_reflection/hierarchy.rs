use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use crate::scene::components::Hierarchy;
use crate::scene::{
    derived_component_registration_with_adapter, EntityId, ReflectComponent,
    RuntimeTypeRegistration, World,
};

use super::component_support;

const TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";

pub(super) fn registration() -> Result<RuntimeTypeRegistration, ReflectError> {
    derived_component_registration_with_adapter::<Hierarchy>(
        ReflectComponent::new(TYPE_PATH, contains, read_field, write_field, remove)
            .with_dense_field_slots(read_field_by_slot, write_field_by_slot)
            .with_dense_field_batch_write(write_fields_by_slot),
    )
}

fn contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<Hierarchy>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    match field_name {
        "parent" => read_parent(world, entity),
        _ => Err(component_support::unknown(TYPE_PATH, field_name)),
    }
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if field_name != "parent" {
        return Err(component_support::unknown(TYPE_PATH, field_name));
    }
    write_parent(world, entity, value)
}

fn read_parent(world: &World, entity: EntityId) -> Result<ReflectedValue, ReflectError> {
    component_support::get::<Hierarchy>(world, entity, TYPE_PATH)?;
    Ok(ReflectedValue::Entity(world.parent_of(entity)))
}

fn write_parent(
    world: &mut World,
    entity: EntityId,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let ReflectedValue::Entity(parent) = value else {
        return Err(ReflectError::TypeMismatch {
            type_path: TYPE_PATH.to_string(),
            field_name: "parent".to_string(),
            expected: "Entity".to_string(),
            actual: value.type_name().to_string(),
        });
    };
    match world.set_parent_checked(entity, parent) {
        Ok(changed) => Ok(changed),
        Err(error) => Err(ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("{TYPE_PATH}.parent"),
        }),
    }
}

fn read_field_by_slot(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    match field_slot {
        0 => read_parent(world, entity),
        _ => Err(component_support::unknown(
            TYPE_PATH,
            &format!("#{field_slot}"),
        )),
    }
}

fn write_field_by_slot(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    match field_slot {
        0 => write_parent(world, entity, value),
        _ => Err(component_support::unknown(
            TYPE_PATH,
            &format!("#{field_slot}"),
        )),
    }
}

fn write_fields_by_slot(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    fields: Vec<(u32, ReflectedValue)>,
) -> Result<bool, ReflectError> {
    let mut parent = None;
    for (field_slot, value) in fields {
        if field_slot != 0 {
            return Err(component_support::unknown(
                TYPE_PATH,
                &format!("#{field_slot}"),
            ));
        }
        parent = Some(value);
    }
    match parent {
        Some(value) => write_parent(world, entity, value),
        None => Ok(false),
    }
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    component_support::remove::<Hierarchy>(world, entity, TYPE_PATH)
}
