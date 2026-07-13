use zircon_runtime_interface::reflect::ReflectError;

use crate::scene::ecs::Component;
use crate::scene::{EntityId, World};

pub(super) fn get<'world, T>(
    world: &'world World,
    entity: EntityId,
    type_path: &str,
) -> Result<&'world T, ReflectError>
where
    T: Component,
{
    if !world.contains_entity(entity) {
        return Err(ReflectError::MissingEntity { entity });
    }
    let Some(component) = world.get::<T>(entity) else {
        return Err(missing(entity, type_path));
    };
    Ok(component)
}

pub(super) fn remove<T>(
    world: &mut World,
    entity: EntityId,
    type_path: &str,
) -> Result<bool, ReflectError>
where
    T: Component,
{
    get::<T>(world, entity, type_path)?;
    match world.remove::<T>(entity) {
        Ok(Some(_)) => Ok(true),
        Ok(None) | Err(_) => Err(missing(entity, type_path)),
    }
}

pub(super) fn missing(entity: EntityId, type_path: &str) -> ReflectError {
    ReflectError::MissingComponent {
        entity,
        type_path: type_path.to_string(),
    }
}

pub(super) fn unknown(type_path: &str, field_name: &str) -> ReflectError {
    ReflectError::UnknownField {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
    }
}
