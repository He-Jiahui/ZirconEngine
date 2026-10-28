use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use super::super::AnimationPlayerComponent;

pub(in crate::scene::components::scene) fn read_clip(
    component: &AnimationPlayerComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Resource(component.clip.id().to_string()))
}
