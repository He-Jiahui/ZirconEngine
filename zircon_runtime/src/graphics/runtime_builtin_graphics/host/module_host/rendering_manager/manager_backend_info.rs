use crate::core::framework::render::{
    RenderingBackendInfo, RenderingManager as RenderingManagerFacade,
};

use super::wgpu_rendering_manager::WgpuRenderingManager;

impl RenderingManagerFacade for WgpuRenderingManager {
    fn backend_info(&self) -> RenderingBackendInfo {
        RenderingBackendInfo {
            backend_name: "wgpu".to_string(),
            supports_runtime_preview: true,
            supports_shared_texture_viewports: true,
        }
    }
}
