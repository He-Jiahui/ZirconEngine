#[path = "capacity.rs"]
mod capacity;
#[path = "../../probe_buffer/gpu_layout.rs"]
mod gpu_layout;
#[path = "report.rs"]
mod report;
#[path = "resources.rs"]
mod resources;
#[path = "selection.rs"]
mod selection;
#[path = "../../probe_buffer/slot_allocator.rs"]
mod slot_allocator;
#[path = "upload.rs"]
mod upload;

pub(in crate::graphics::scene::scene_renderer) use gpu_layout::{
    reflection_probe_bind_group_layout_entries, ReflectionProbeGpuBindings,
};
pub(in crate::graphics::scene::scene_renderer) use resources::{
    SceneReflectionProbeResources, PLANAR_REFLECTION_MIP_COUNT, PLANAR_REFLECTION_TEXTURE_SIZE,
};
pub(in crate::graphics) use slot_allocator::ProbeCubemapSlotReservation;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
