use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldValue, ReflectTypeRegistration, ReflectedValue, ZrReflect,
};

use crate::scene::ecs::Component;
use crate::scene::{EntityId, World};

use crate::scene::reflect::{ReflectComponent, RuntimeTypeRegistration};

pub fn derived_component_registration<T>() -> Result<RuntimeTypeRegistration, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let registration = T::reflect_type_registration()?;
    let type_path = registration.type_path.type_path.clone();
    derived_component_registration_with_adapter::<T>(
        ReflectComponent::new(
            type_path,
            contains::<T>,
            read_field::<T>,
            read_fields::<T>,
            write_field::<T>,
            remove::<T>,
        )
        .with_dense_field_slots(read_field_by_slot::<T>, write_field_by_slot::<T>),
    )
}

pub fn derived_component_registration_with_adapter<T>(
    component: ReflectComponent,
) -> Result<RuntimeTypeRegistration, ReflectError>
where
    T: ZrReflect,
{
    let registration = T::reflect_type_registration()?;
    validate_component_registration(&registration)?;
    Ok(RuntimeTypeRegistration {
        registration,
        component: Some(component),
        resource: None,
    })
}

fn validate_component_registration(
    registration: &ReflectTypeRegistration,
) -> Result<(), ReflectError> {
    if registration.is_component && !registration.is_resource {
        return Ok(());
    }
    Err(ReflectError::InvalidRegistration {
        type_path: registration.type_path.type_path.clone(),
        reason: "derived component adapters require component-only registrations".to_string(),
    })
}

fn contains<T>(world: &World, entity: EntityId, _type_path: &str) -> bool
where
    T: Component + ZrReflect + Clone,
{
    world.get::<T>(entity).is_some()
}

fn read_field<T>(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    component::<T>(world, entity, type_path)?.read_reflected_field(field_name)
}

fn read_fields<T>(
    world: &World,
    entity: EntityId,
    type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let component = component::<T>(world, entity, type_path)?;
    let registration = world.type_registry().registration(type_path)?;
    let mut values = Vec::with_capacity(registration.type_info.fields.len());
    for field in &registration.type_info.fields {
        values.push(ReflectFieldValue::new(
            field.name.clone(),
            component.read_reflected_field(&field.name)?,
        ));
    }
    Ok(values)
}

fn write_field<T>(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let mut next = component::<T>(world, entity, type_path)?.clone();
    if !next.write_reflected_field(field_name, value)? {
        return Ok(false);
    }
    match world.insert(entity, next) {
        Ok(_) => Ok(true),
        Err(error) => Err(ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("{type_path}.{field_name}"),
        }),
    }
}

fn read_field_by_slot<T>(
    world: &World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    component::<T>(world, entity, type_path)?.read_reflected_field_by_slot(field_slot)
}

fn write_field_by_slot<T>(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
    field_slot: u32,
    value: ReflectedValue,
) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    let mut next = component::<T>(world, entity, type_path)?.clone();
    if !next.write_reflected_field_by_slot(field_slot, value)? {
        return Ok(false);
    }
    match world.insert(entity, next) {
        Ok(_) => Ok(true),
        Err(error) => Err(ReflectError::UnsupportedConversion {
            source: error.to_string(),
            target: format!("{type_path}.#{field_slot}"),
        }),
    }
}

fn remove<T>(world: &mut World, entity: EntityId, type_path: &str) -> Result<bool, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    ensure_entity(world, entity)?;
    if world.get::<T>(entity).is_none() {
        return Err(missing_component(entity, type_path));
    }
    match world.remove::<T>(entity) {
        Ok(Some(_)) => Ok(true),
        Ok(None) | Err(_) => Err(missing_component(entity, type_path)),
    }
}

fn component<'a, T>(
    world: &'a World,
    entity: EntityId,
    type_path: &str,
) -> Result<&'a T, ReflectError>
where
    T: Component + ZrReflect + Clone,
{
    ensure_entity(world, entity)?;
    let Some(component) = world.get::<T>(entity) else {
        return Err(missing_component(entity, type_path));
    };
    Ok(component)
}

fn ensure_entity(world: &World, entity: EntityId) -> Result<(), ReflectError> {
    if world.contains_entity(entity) {
        return Ok(());
    }
    Err(ReflectError::MissingEntity { entity })
}

fn missing_component(entity: EntityId, type_path: &str) -> ReflectError {
    ReflectError::MissingComponent {
        entity,
        type_path: type_path.to_string(),
    }
}
