use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

use crate::scene::{
    components::ActiveInHierarchy, reflect::ReflectComponent, reflect::RuntimeTypeRegistration,
    reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::ActiveInHierarchy";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(RuntimeTypeRegistration {
        registration: ReflectTypeRegistration::new(
            ReflectTypePath::new(TYPE_PATH, "ActiveInHierarchy")
                .expect("fixed component reflection type paths must be valid"),
            "ActiveInHierarchy",
            ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                "value",
                "Bool",
                ReflectEditorHint::Bool,
            )
            .with_editable(false)
            .with_serializable(false)]),
            ReflectSerializationStrategy::None,
        )
        .as_component(),
        component: Some(adapter()),
        resource: None,
    })
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
    world.get::<ActiveInHierarchy>(entity).is_some() || world.contains_entity(entity)
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    match field_name {
        "value" => world
            .active_in_hierarchy(entity)
            .map(ReflectedValue::Bool)
            .ok_or_else(|| shared::missing_component(entity, TYPE_PATH)),
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
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    field_name: &str,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    Err(shared::non_editable_field(TYPE_PATH, field_name))
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<ActiveInHierarchy>(world, entity, TYPE_PATH)
}
