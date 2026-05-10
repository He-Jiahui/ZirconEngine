use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::scene::{
    components::ActiveSelf, reflect::ReflectComponent, reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::ActiveSelf";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        TYPE_PATH,
        "ActiveSelf",
        vec![shared::field("value", "Bool", ReflectEditorHint::Bool)],
        adapter(),
    ))
}

fn adapter() -> ReflectComponent {
    ReflectComponent::new(
        TYPE_PATH,
        contains,
        read_field,
        read_fields,
        write_field,
        remove,
    )
}

fn contains(world: &World, entity: EntityId, _type_path: &str) -> bool {
    world.get::<ActiveSelf>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let component = shared::get_component::<ActiveSelf>(world, entity, TYPE_PATH)?;
    match field_name {
        "value" => Ok(ReflectedValue::Bool(component.0)),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "value",
        read_field(world, entity, TYPE_PATH, "value")?,
    )])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let current = shared::get_component::<ActiveSelf>(world, entity, TYPE_PATH)?;
    if field_name != "value" {
        return Err(shared::unknown_field(TYPE_PATH, field_name));
    }
    let next = shared::expect_bool(TYPE_PATH, field_name, value)?;
    if current.0 == next {
        return Ok(false);
    }
    world
        .insert(entity, ActiveSelf(next))
        .map_err(|_| shared::missing_component(entity, TYPE_PATH))?;
    Ok(true)
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<ActiveSelf>(world, entity, TYPE_PATH)
}
