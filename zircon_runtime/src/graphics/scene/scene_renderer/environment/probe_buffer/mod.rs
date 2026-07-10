mod gpu_layout;
mod resources;
mod slot_allocator;
mod upload;

pub(in crate::graphics::scene::scene_renderer) use gpu_layout::{
    reflection_probe_bind_group_layout_entries, ReflectionProbeGpuBindings,
};
pub(in crate::graphics::scene::scene_renderer) use resources::SceneReflectionProbeResources;

#[cfg(test)]
mod tests;
