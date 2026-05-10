use std::sync::Mutex;

#[cfg(test)]
use crate::core::framework::render::RenderCapabilitySummary;

use super::super::render_framework_state::RenderFrameworkState;

pub struct WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) state: Mutex<RenderFrameworkState>,
    pub(in crate::graphics::runtime::render_framework) operation_lock: Mutex<()>,
}

impl WgpuRenderFramework {
    #[cfg(test)]
    pub(crate) fn override_capabilities_for_tests(&self, capabilities: RenderCapabilitySummary) {
        self.state.lock().unwrap().stats.capabilities = capabilities;
    }

    #[cfg(test)]
    pub(crate) fn request_next_created_viewport_graphics_debugger_capture_for_tests(&self) {
        self.state
            .lock()
            .unwrap()
            .graphics_debugger
            .request_next_created_viewport_capture();
    }
}
