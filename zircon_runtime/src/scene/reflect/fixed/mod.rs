mod active_in_hierarchy;
mod active_self;
mod camera_component;
mod hierarchy;
mod lights;
mod local_transform;
mod mesh_renderer;
mod mobility;
mod name;
mod render_layer_mask;
mod rigid_body_component;
mod shared;

use zircon_runtime_interface::reflect::ReflectError;

use crate::scene::reflect::TypeRegistry;

pub(super) fn register_fixed_components(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    name::register(registry)?;
    hierarchy::register(registry)?;
    local_transform::register(registry)?;
    active_self::register(registry)?;
    active_in_hierarchy::register(registry)?;
    render_layer_mask::register(registry)?;
    mobility::register(registry)?;
    camera_component::register(registry)?;
    mesh_renderer::register(registry)?;
    lights::register(registry)?;
    rigid_body_component::register(registry)?;
    Ok(())
}
