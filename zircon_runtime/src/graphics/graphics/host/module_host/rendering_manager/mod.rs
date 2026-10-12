#[path = "../../../../host/rendering_manager/manager_backend_info.rs"]
mod manager_backend_info;
#[path = "../../../../host/rendering_manager/wgpu_rendering_manager.rs"]
mod wgpu_rendering_manager;

pub(crate) use wgpu_rendering_manager::WgpuRenderingManager;
