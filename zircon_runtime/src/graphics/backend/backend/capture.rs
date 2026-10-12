use crate::graphics::types::GraphicsError;

use super::render_backend::RenderBackend;

pub(crate) struct GraphicsDebuggerCaptureStop {
    device: wgpu::Device,
}

impl GraphicsDebuggerCaptureStop {
    pub(crate) fn stop(self) -> Result<(), GraphicsError> {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| GraphicsError::GraphicsDebugger(error.to_string()))?;
        unsafe { self.device.stop_graphics_debugger_capture() };
        Ok(())
    }
}

impl RenderBackend {
    pub(crate) fn backend_name(&self) -> &str {
        &self.backend_name
    }

    pub(crate) fn start_graphics_debugger_capture(&self) {
        // wgpu forwards this to RenderDoc/Xcode when attached; otherwise it is a no-op.
        unsafe { self.device.start_graphics_debugger_capture() };
    }

    pub(crate) fn prepare_graphics_debugger_capture_stop(&self) -> GraphicsDebuggerCaptureStop {
        GraphicsDebuggerCaptureStop {
            device: self.device.clone(),
        }
    }
}
