use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::scene::{
    components::Mobility, reflect::ReflectComponent, reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::core::framework::scene::Mobility";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        TYPE_PATH,
        "Mobility",
        vec![shared::field("kind", "Enum", ReflectEditorHint::Enum)],
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
    world.get::<Mobility>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let mobility = shared::get_component::<Mobility>(world, entity, TYPE_PATH)?;
    match field_name {
        "kind" => Ok(ReflectedValue::Enum(mobility_label(*mobility).to_string())),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "kind",
        read_field(world, entity, TYPE_PATH, "kind")?,
    )])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    if field_name != "kind" {
        return Err(shared::unknown_field(TYPE_PATH, field_name));
    }
    let kind = match value {
        ReflectedValue::Enum(kind) => kind,
        value => {
            return Err(ReflectError::TypeMismatch {
                type_path: TYPE_PATH.to_string(),
                field_name: field_name.to_string(),
                expected: "Enum".to_string(),
                actual: value.type_name().to_string(),
            });
        }
    };
    world
        .set_mobility(entity, parse_mobility(&kind)?)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error,
            target: format!("{TYPE_PATH}.{field_name}"),
        })
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<Mobility>(world, entity, TYPE_PATH)
}

fn mobility_label(mobility: Mobility) -> &'static str {
    match mobility {
        Mobility::Dynamic => "dynamic",
        Mobility::Static => "static",
    }
}

fn parse_mobility(value: &str) -> Result<Mobility, ReflectError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "dynamic" => Ok(Mobility::Dynamic),
        "static" => Ok(Mobility::Static),
        _ => Err(ReflectError::UnsupportedConversion {
            source: value.to_string(),
            target: "Mobility".to_string(),
        }),
    }
}
