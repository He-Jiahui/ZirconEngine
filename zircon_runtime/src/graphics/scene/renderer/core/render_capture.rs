use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::backend::GraphicsDebuggerCaptureStop;
use crate::graphics::resource_limits::OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
use crate::graphics::types::GraphicsError;
use crate::rhi::RenderBackendCaps;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub fn backend_name(&self) -> &str {
        self.backend.backend_name()
    }

    pub(crate) fn backend_caps(&self) -> RenderBackendCaps {
        self.backend.caps()
    }

    /// Qualifies product passes recorded by this renderer's actual native device.
    /// All limits remain the negotiated device limits from the neutral capability receipt.
    pub(crate) fn native_render_capabilities(&self) -> RenderCapabilitySummary {
        let mut capabilities = crate::graphics::runtime::capability_summary(&self.backend_caps());
        let native_fragment_writable_storage =
            self.backend.native_fragment_writable_storage_supported();
        let native_oit_bindings = self.backend.device.limits().max_bind_groups >= 5
            && capabilities.storage_buffer_binding_capacity_supported(
                OIT_MESH_PIPELINE_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            );
        capabilities.supports_fragment_writable_storage =
            native_fragment_writable_storage && native_oit_bindings;
        capabilities
    }

    /// 由产品调试入口显式开始捕获；捕获边界由 backend 持有，与 scene 图的帧提交分离。
    pub fn start_graphics_debugger_capture(&self) {
        self.backend.start_graphics_debugger_capture();
    }

    pub fn stop_graphics_debugger_capture(&self) -> Result<(), GraphicsError> {
        self.prepare_graphics_debugger_capture_stop().stop()
    }

    pub(crate) fn prepare_graphics_debugger_capture_stop(&self) -> GraphicsDebuggerCaptureStop {
        self.backend.prepare_graphics_debugger_capture_stop()
    }
}
