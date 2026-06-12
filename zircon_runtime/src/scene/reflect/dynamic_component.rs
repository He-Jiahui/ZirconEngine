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
    let mut fields = Vec::with_capacity(descriptor.properties.len());
    for property in &descriptor.properties {
        fields.push(field_from_property_descriptor(
            &descriptor.type_id,
            property,
        )?);
    }

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
    .with_editable(descriptor.editable))
}

fn short_type_path(type_path: &str) -> &str {
    if let Some((_, short)) = type_path.rsplit_once('.') {
        return short;
    }

    type_path
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
    let Some(value) = world.dynamic_component_property(entity, &property_path) else {
        return Err(ReflectError::UnsupportedConversion {
            source: format!("dynamic JSON property `{type_path}.{field_name}`"),
            target: "ReflectedValue".to_string(),
        });
    };

    reflected_from_scene_value(value)
}

fn read_fields(
    world: &World,
    entity: EntityId,
    type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    let registration = world.type_registry().registration(type_path)?;
    let fields = &registration.type_info.fields;
    let mut values = Vec::with_capacity(fields.len());
    for field in fields {
        let value = read_field(world, entity, type_path, &field.name)?;
        values.push(ReflectFieldValue::new(field.name.clone(), value));
    }
    Ok(values)
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

    let Some(component) = world.dynamic_component(entity, type_path) else {
        return Err(ReflectError::MissingComponent {
            entity,
            type_path: type_path.to_string(),
        });
    };

    Ok(component)
}

fn ensure_declared_field<'a>(
    registration: &'a ReflectTypeRegistration,
    field_name: &str,
) -> Result<&'a ReflectFieldInfo, ReflectError> {
    for field in &registration.type_info.fields {
        if field.name == field_name {
            return Ok(field);
        }
    }

    Err(ReflectError::UnknownField {
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
    let Some(object) = component.as_object() else {
        return Err(ReflectError::UnknownField {
            type_path: type_path.to_string(),
            field_name: field_name.to_string(),
        });
    };

    if object.contains_key(field_name) {
        return Ok(());
    }

    Err(ReflectError::UnknownField {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
    })
}

fn dynamic_property_path(
    type_path: &str,
    field_name: &str,
) -> Result<ComponentPropertyPath, ReflectError> {
    let mut property_segments = Vec::with_capacity(1);
    property_segments.push(field_name.to_string());

    ComponentPropertyPath::new(type_path.to_string(), property_segments).map_err(|error| {
        ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: error.to_string(),
        }
    })
}
