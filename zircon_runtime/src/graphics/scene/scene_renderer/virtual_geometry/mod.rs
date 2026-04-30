mod gpu_readback;
mod gpu_resources;

pub(in crate::graphics::scene::scene_renderer) use gpu_readback::VirtualGeometryGpuPendingReadback;
pub(crate) use gpu_readback::VirtualGeometryGpuReadback;
pub(in crate::graphics) use gpu_readback::VirtualGeometryGpuReadbackCompletionParts;
pub(in crate::graphics::scene::scene_renderer) use gpu_resources::VirtualGeometryGpuResources;
