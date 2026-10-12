#[path = "collect.rs"]
mod collect;
#[path = "new.rs"]
mod new;
#[path = "../pending_readback_mod.rs"]
mod virtual_geometry_gpu_pending_readback;

pub(crate) use collect::VirtualGeometryGpuReadbackFuture;
pub(crate) use virtual_geometry_gpu_pending_readback::VirtualGeometryGpuPendingReadback;
