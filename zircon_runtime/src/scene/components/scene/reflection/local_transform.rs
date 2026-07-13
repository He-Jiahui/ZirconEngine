use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue, ZrReflectValue};

use crate::core::math::Vec3;

use super::super::LocalTransform;

const TYPE_PATH: &str = "zircon_runtime::scene::components::LocalTransform";

pub(in crate::scene::components::scene) fn read_translation(
    component: &LocalTransform,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component.transform.translation.to_reflected_value())
}

pub(in crate::scene::components::scene) fn write_translation(
    component: &mut LocalTransform,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let next = Vec3::from_reflected_value(value, TYPE_PATH, "translation")?;
    if component.transform.translation == next {
        return Ok(false);
    }
    component.transform.translation = next;
    Ok(true)
}

pub(in crate::scene::components::scene) fn read_rotation(
    component: &LocalTransform,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Vec4(
        component.transform.rotation.to_array(),
    ))
}

pub(in crate::scene::components::scene) fn read_scale(
    component: &LocalTransform,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component.transform.scale.to_reflected_value())
}

pub(in crate::scene::components::scene) fn write_scale(
    component: &mut LocalTransform,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let next = Vec3::from_reflected_value(value, TYPE_PATH, "scale")?;
    if component.transform.scale == next {
        return Ok(false);
    }
    component.transform.scale = next;
    Ok(true)
}
