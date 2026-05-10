use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::core::math::Vec3;
use crate::scene::{
    components::LocalTransform, reflect::ReflectComponent, reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::LocalTransform";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        TYPE_PATH,
        "LocalTransform",
        vec![
            shared::field("translation", "Vec3", ReflectEditorHint::Vec3),
            shared::readonly_field("rotation", "Vec4", ReflectEditorHint::Vec4),
            shared::field("scale", "Vec3", ReflectEditorHint::Vec3),
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
    world.get::<LocalTransform>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let component = shared::get_component::<LocalTransform>(world, entity, TYPE_PATH)?;
    match field_name {
        "translation" => Ok(ReflectedValue::Vec3(
            component.transform.translation.to_array(),
        )),
        "rotation" => Ok(ReflectedValue::Vec4(
            component.transform.rotation.to_array(),
        )),
        "scale" => Ok(ReflectedValue::Vec3(component.transform.scale.to_array())),
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
            "translation",
            read_field(world, entity, TYPE_PATH, "translation")?,
        ),
        ReflectFieldValue::new(
            "rotation",
            read_field(world, entity, TYPE_PATH, "rotation")?,
        ),
        ReflectFieldValue::new("scale", read_field(world, entity, TYPE_PATH, "scale")?),
    ])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<LocalTransform>(world, entity, TYPE_PATH)?;
    match field_name {
        "translation" => write_translation(world, entity, value),
        "scale" => write_scale(world, entity, value),
        "rotation" => Err(shared::non_editable_field(TYPE_PATH, field_name)),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn write_translation(
    world: &mut World,
    entity: EntityId,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let next = Vec3::from_array(shared::expect_vec3(TYPE_PATH, "translation", value)?);
    if shared::get_component::<LocalTransform>(world, entity, TYPE_PATH)?
        .transform
        .translation
        == next
    {
        return Ok(false);
    }
    let component = shared::get_component_mut::<LocalTransform>(world, entity, TYPE_PATH)?;
    component.transform.translation = next;
    Ok(true)
}

fn write_scale(
    world: &mut World,
    entity: EntityId,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let next = Vec3::from_array(shared::expect_vec3(TYPE_PATH, "scale", value)?);
    if shared::get_component::<LocalTransform>(world, entity, TYPE_PATH)?
        .transform
        .scale
        == next
    {
        return Ok(false);
    }
    let component = shared::get_component_mut::<LocalTransform>(world, entity, TYPE_PATH)?;
    component.transform.scale = next;
    Ok(true)
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<LocalTransform>(world, entity, TYPE_PATH)
}
