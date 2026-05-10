use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::scene::{
    components::RenderLayerMask, reflect::ReflectComponent, reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::RenderLayerMask";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        TYPE_PATH,
        "RenderLayerMask",
        vec![shared::field(
            "mask",
            "Unsigned",
            ReflectEditorHint::Unsigned,
        )],
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
    world.get::<RenderLayerMask>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let component = shared::get_component::<RenderLayerMask>(world, entity, TYPE_PATH)?;
    match field_name {
        "mask" => Ok(ReflectedValue::Unsigned(component.0 as u64)),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "mask",
        read_field(world, entity, TYPE_PATH, "mask")?,
    )])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let current = shared::get_component::<RenderLayerMask>(world, entity, TYPE_PATH)?;
    if field_name != "mask" {
        return Err(shared::unknown_field(TYPE_PATH, field_name));
    }
    let next = match value {
        ReflectedValue::Unsigned(value) => value,
        value => {
            return Err(shared::type_mismatch(
                TYPE_PATH, field_name, "Unsigned", &value,
            ));
        }
    };
    if next > u32::MAX as u64 {
        return Err(ReflectError::TypeMismatch {
            type_path: TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "Unsigned <= u32::MAX".to_string(),
            actual: next.to_string(),
        });
    }
    let next = next as u32;
    if current.0 == next {
        return Ok(false);
    }
    world
        .insert(entity, RenderLayerMask(next))
        .map_err(|_| shared::missing_component(entity, TYPE_PATH))?;
    Ok(true)
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<RenderLayerMask>(world, entity, TYPE_PATH)
}
