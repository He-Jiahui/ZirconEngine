use crate::core::framework::scene::ComponentPropertyPath;
use crate::plugin::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
use crate::scene::{
    reflect::{reflected_from_scene_value, scene_value_from_reflected, ReflectComponent},
    EntityId, World,
};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

pub fn registration_from_component_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> Result<ReflectTypeRegistration, ReflectError> {
    let fields = descriptor
        .properties
        .iter()
        .map(|property| field_from_property_descriptor(&descriptor.type_id, property))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ReflectTypeRegistration::new(
        ReflectTypePath::new(
            descriptor.type_id.clone(),
            short_type_path(&descriptor.type_id).to_string(),
        )?,
        descriptor.display_name.clone(),
        ReflectTypeInfo::json_with_fields(fields),
        ReflectSerializationStrategy::Json,
    )
    .as_component()
    .with_plugin_owned(true)
    .with_serializable(true)
    .with_editor_visible(true)
    .with_remote_visible(true)
    .with_plugin_id(descriptor.plugin_id.clone()))
}

pub fn reflect_component_for_dynamic_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> ReflectComponent {
    ReflectComponent::new(
        descriptor.type_id.clone(),
        contains,
        read_field,
        read_fields,
        write_field,
        remove,
    )
}

fn field_from_property_descriptor(
    type_path: &str,
    descriptor: &ComponentPropertyDescriptor,
) -> Result<ReflectFieldInfo, ReflectError> {
    if descriptor.name.trim().is_empty() {
        return Err(ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: "dynamic component field name must not be empty".to_string(),
        });
    }
    if descriptor.value_type.trim().is_empty() {
        return Err(ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: format!(
                "dynamic component field `{}` value type must not be empty",
                descriptor.name
            ),
        });
    }

    Ok(ReflectFieldInfo::new(
        descriptor.name.clone(),
        descriptor.value_type.clone(),
        ReflectEditorHint::None,
    )
    .with_display_name(descriptor.name.clone())
    .with_editable(descriptor.editable))
}

fn short_type_path(type_path: &str) -> &str {
    type_path
        .rsplit_once('.')
        .map(|(_, short)| short)
        .unwrap_or(type_path)
}

fn contains(world: &World, entity: EntityId, type_path: &str) -> bool {
    world.contains_entity(entity) && world.dynamic_component(entity, type_path).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let registration = world.type_registry().registration(type_path)?;
    ensure_declared_field(registration, field_name)?;
    ensure_json_field_present(world, entity, type_path, field_name)?;
    let property_path = dynamic_property_path(type_path, field_name)?;
    let value = world
        .dynamic_component_property(entity, &property_path)
        .ok_or_else(|| ReflectError::UnsupportedConversion {
            source: format!("dynamic JSON property `{type_path}.{field_name}`"),
            target: "ReflectedValue".to_string(),
        })?;

    reflected_from_scene_value(value)
}

fn read_fields(
    world: &World,
    entity: EntityId,
    type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    let registration = world.type_registry().registration(type_path)?;
    registration
        .type_info
        .fields
        .iter()
        .map(|field| {
            read_field(world, entity, type_path, &field.name)
                .map(|value| ReflectFieldValue::new(field.name.clone(), value))
        })
        .collect()
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let registration = world.type_registry().registration(type_path)?;
    let field = ensure_declared_field(registration, field_name)?;
    if !field.editable {
        return Err(ReflectError::NonEditableField {
            type_path: type_path.to_string(),
            field_name: field_name.to_string(),
        });
    }
    ensure_dynamic_component(world, entity, type_path)?;

    let property_path = dynamic_property_path(type_path, field_name)?;
    let value = scene_value_from_reflected(value)?;
    world
        .set_dynamic_component_property(entity, &property_path, value)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error,
            target: format!("dynamic JSON property `{type_path}.{field_name}`"),
        })
}

fn remove(world: &mut World, entity: EntityId, type_path: &str) -> Result<bool, ReflectError> {
    ensure_dynamic_component(world, entity, type_path)?;
    world
        .remove_dynamic_component(entity, type_path)
        .map_err(|error| ReflectError::UnsupportedConversion {
            source: error,
            target: format!("dynamic component `{type_path}` removal"),
        })
}

fn ensure_dynamic_component<'a>(
    world: &'a World,
    entity: EntityId,
    type_path: &str,
) -> Result<&'a serde_json::Value, ReflectError> {
    if !world.contains_entity(entity) {
        return Err(ReflectError::MissingEntity { entity });
    }
    world
        .dynamic_component(entity, type_path)
        .ok_or_else(|| ReflectError::MissingComponent {
            entity,
            type_path: type_path.to_string(),
        })
}

fn ensure_declared_field<'a>(
    registration: &'a ReflectTypeRegistration,
    field_name: &str,
) -> Result<&'a ReflectFieldInfo, ReflectError> {
    registration
        .type_info
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .ok_or_else(|| ReflectError::UnknownField {
            type_path: registration.type_path.type_path.clone(),
            field_name: field_name.to_string(),
        })
}

fn ensure_json_field_present(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
) -> Result<(), ReflectError> {
    let component = ensure_dynamic_component(world, entity, type_path)?;
    component
        .as_object()
        .and_then(|object| object.get(field_name))
        .map(|_| ())
        .ok_or_else(|| ReflectError::UnknownField {
            type_path: type_path.to_string(),
            field_name: field_name.to_string(),
        })
}

fn dynamic_property_path(
    type_path: &str,
    field_name: &str,
) -> Result<ComponentPropertyPath, ReflectError> {
    ComponentPropertyPath::parse(&format!("{type_path}.{field_name}")).map_err(|error| {
        ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: error.to_string(),
        }
    })
}
