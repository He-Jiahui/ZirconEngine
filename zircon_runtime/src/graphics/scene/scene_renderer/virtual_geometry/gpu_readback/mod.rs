mod decode;
mod pending_readback;
mod readback;

pub(in crate::graphics::scene::scene_renderer) use pending_readback::VirtualGeometryGpuPendingReadback;
pub(crate) use readback::VirtualGeometryGpuReadback;
pub(in crate::graphics) use readback::VirtualGeometryGpuReadbackCompletionParts;
