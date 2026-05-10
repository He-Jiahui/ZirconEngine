use crate::graphics::backend::GraphicsDebuggerCaptureStop;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn backend_name(&self) -> &str {
        self.backend.backend_name()
    }

    pub(crate) fn start_graphics_debugger_capture(&self) {
        self.backend.start_graphics_debugger_capture();
    }

    pub(crate) fn prepare_graphics_debugger_capture_stop(&self) -> GraphicsDebuggerCaptureStop {
        self.backend.prepare_graphics_debugger_capture_stop()
    }
}
