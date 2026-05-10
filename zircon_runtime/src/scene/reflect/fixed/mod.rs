mod active_self;
mod local_transform;
mod name;
mod render_layer_mask;
mod rigid_body_component;
mod shared;

use zircon_runtime_interface::reflect::ReflectError;

use crate::scene::reflect::TypeRegistry;

pub(super) fn register_fixed_components(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    name::register(registry)?;
    local_transform::register(registry)?;
    active_self::register(registry)?;
    render_layer_mask::register(registry)?;
    rigid_body_component::register(registry)?;
    Ok(())
}
