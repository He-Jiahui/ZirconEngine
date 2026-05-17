use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::scene::{
    components::Hierarchy, reflect::ReflectComponent, reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    let mut registration = shared::component_registration(
        TYPE_PATH,
        "Hierarchy",
        vec![shared::field("parent", "Entity", ReflectEditorHint::Entity).with_serializable(false)],
        adapter(),
    );
    registration.registration.serializable = false;
    registry.register(registration)
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
    world.get::<Hierarchy>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    shared::ensure_component::<Hierarchy>(world, entity, TYPE_PATH)?;
    match field_name {
        "parent" => Ok(ReflectedValue::Entity(world.parent_of(entity))),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "parent",
        read_field(world, entity, TYPE_PATH, "parent")?,
    )])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if field_name != "parent" {
        return Err(shared::unknown_field(TYPE_PATH, field_name));
    }
    let parent = match value {
        ReflectedValue::Entity(parent) => parent,
        value => {
            return Err(shared::type_mismatch(
                TYPE_PATH, field_name, "Entity", &value,
            ))
        }
    };
    world
        .set_parent_checked(entity, parent)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error,
            target: format!("{TYPE_PATH}.{field_name}"),
        })
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<Hierarchy>(world, entity, TYPE_PATH)
}
