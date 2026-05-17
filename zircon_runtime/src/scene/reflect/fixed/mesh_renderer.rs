use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldValue, ReflectedValue,
};

use crate::core::math::Vec4;
use crate::scene::{
    components::MeshRenderer, reflect::ReflectComponent, reflect::TypeRegistry, EntityId, World,
};

use super::shared;

pub(super) const TYPE_PATH: &str = "zircon_runtime::scene::components::MeshRenderer";

pub(super) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(shared::component_registration(
        TYPE_PATH,
        "MeshRenderer",
        vec![
            shared::readonly_field("model", "Resource", ReflectEditorHint::Resource),
            shared::readonly_field("material", "Resource", ReflectEditorHint::Resource),
            shared::field("tint", "Vec4", ReflectEditorHint::Vec4),
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
    world.get::<MeshRenderer>(entity).is_some()
}

fn read_field(
    world: &World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let component = shared::get_component::<MeshRenderer>(world, entity, TYPE_PATH)?;
    match field_name {
        "model" => Ok(ReflectedValue::Resource(component.model.id().to_string())),
        "material" => Ok(ReflectedValue::Resource(
            component.material.id().to_string(),
        )),
        "tint" => Ok(ReflectedValue::Vec4(component.tint.to_array())),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn read_fields(
    world: &World,
    entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![
        ReflectFieldValue::new("model", read_field(world, entity, TYPE_PATH, "model")?),
        ReflectFieldValue::new(
            "material",
            read_field(world, entity, TYPE_PATH, "material")?,
        ),
        ReflectFieldValue::new("tint", read_field(world, entity, TYPE_PATH, "tint")?),
    ])
}

fn write_field(
    world: &mut World,
    entity: EntityId,
    _type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    shared::ensure_component::<MeshRenderer>(world, entity, TYPE_PATH)?;
    match field_name {
        "tint" => {
            let next = Vec4::from_array(shared::expect_vec4(TYPE_PATH, field_name, value)?);
            if shared::get_component::<MeshRenderer>(world, entity, TYPE_PATH)?.tint == next {
                return Ok(false);
            }
            shared::get_component_mut::<MeshRenderer>(world, entity, TYPE_PATH)?.tint = next;
            Ok(true)
        }
        "model" | "material" => Err(shared::non_editable_field(TYPE_PATH, field_name)),
        _ => Err(shared::unknown_field(TYPE_PATH, field_name)),
    }
}

fn remove(world: &mut World, entity: EntityId, _type_path: &str) -> Result<bool, ReflectError> {
    shared::remove_component::<MeshRenderer>(world, entity, TYPE_PATH)
}
