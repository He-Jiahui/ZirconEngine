use crate::graphics::backend::GraphicsDebuggerCaptureStop;
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
