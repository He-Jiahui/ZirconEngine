use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::scene::{
    components::CameraComponent, reflect::ReflectComponent, reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::CameraComponent";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        TYPE_PATH,
        "CameraComponent",
        vec![
            shared::field("fov_y_radians", "Scalar", ReflectEditorHint::Scalar),
            shared::field("z_near", "Scalar", ReflectEditorHint::Scalar),
            shared::field("z_far", "Scalar", ReflectEditorHint::Scalar),
        ],
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
    world.get::<CameraComponent>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let component = shared::get_component::<CameraComponent>(world, entity, TYPE_PATH)?;
    match field_name {
        "fov_y_radians" => Ok(ReflectedValue::Scalar(component.fov_y_radians)),
        "z_near" => Ok(ReflectedValue::Scalar(component.z_near)),
        "z_far" => Ok(ReflectedValue::Scalar(component.z_far)),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new(
            "fov_y_radians",
            read_field(world, entity, TYPE_PATH, "fov_y_radians")?,
        ),
        ReflectFieldValue::new("z_near", read_field(world, entity, TYPE_PATH, "z_near")?),
        ReflectFieldValue::new("z_far", read_field(world, entity, TYPE_PATH, "z_far")?),
    ])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<CameraComponent>(world, entity, TYPE_PATH)?;
    let next = shared::expect_scalar(TYPE_PATH, field_name, value)?;
    let component = shared::get_component::<CameraComponent>(world, entity, TYPE_PATH)?;
    let current = match field_name {
        "fov_y_radians" => component.fov_y_radians,
        "z_near" => component.z_near,
        "z_far" => component.z_far,
        _ => return Err(shared::unknown_field(TYPE_PATH, field_name)),
    };
    if current == next {
        return Ok(false);
    }
    let component = shared::get_component_mut::<CameraComponent>(world, entity, TYPE_PATH)?;
    match field_name {
        "fov_y_radians" => component.fov_y_radians = next,
        "z_near" => component.z_near = next,
        "z_far" => component.z_far = next,
        _ => return Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
    Ok(true)
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<CameraComponent>(world, entity, TYPE_PATH)
}
