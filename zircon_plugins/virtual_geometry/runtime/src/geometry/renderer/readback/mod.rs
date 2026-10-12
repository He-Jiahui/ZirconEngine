#[path = "../../readback/readback/mod.rs"]
mod virtual_geometry_gpu_readback;
#[path = "../../../virtual_geometry_gpu_readback.rs"]
mod virtual_geometry_gpu_readback_completion_parts;

pub(crate) use virtual_geometry_gpu_readback::VirtualGeometryGpuReadback;
pub(crate) use virtual_geometry_gpu_readback_completion_parts::VirtualGeometryGpuReadbackCompletionParts;
