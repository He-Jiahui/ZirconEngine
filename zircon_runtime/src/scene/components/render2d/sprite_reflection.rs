use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use super::Sprite2dComponent;

pub(super) fn read_image(component: &Sprite2dComponent) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Resource(component.image.id().to_string()))
}

pub(super) fn read_material(component: &Sprite2dComponent) -> Result<ReflectedValue, ReflectError> {
    Ok(component.material.map_or(ReflectedValue::Null, |material| {
        ReflectedValue::Resource(material.id().to_string())
    }))
}

pub(super) fn read_custom_size(
    component: &Sprite2dComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component.custom_size.map_or(ReflectedValue::Null, |size| {
        ReflectedValue::Vec2(size.to_array())
    }))
}
