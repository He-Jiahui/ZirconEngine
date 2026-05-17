use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
use crate::core::framework::render::RenderCapabilitySummary;

use super::super::render_framework_state::RenderFrameworkState;

pub struct WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) state: Mutex<RenderFrameworkState>,
    pub(in crate::graphics::runtime::render_framework) operation_lock: Mutex<()>,
}

impl WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) fn lock_operation(
        &self,
    ) -> MutexGuard<'_, ()> {
        #[cfg(feature = "profiling")]
        crate::profile_scope!("runtime", "render_framework.wait", "operation_lock");
        self.operation_lock.lock().unwrap()
    }

    pub(in crate::graphics::runtime::render_framework) fn lock_state(
        &self,
    ) -> MutexGuard<'_, RenderFrameworkState> {
        #[cfg(feature = "profiling")]
        crate::profile_scope!("runtime", "render_framework.wait", "state");
        self.state.lock().unwrap()
    }

    #[cfg(test)]
    pub(crate) fn override_capabilities_for_tests(&self, capabilities: RenderCapabilitySummary) {
        self.lock_state().stats.capabilities = capabilities;
    }

    #[cfg(test)]
    pub(crate) fn request_next_created_viewport_graphics_debugger_capture_for_tests(&self) {
        self.lock_state()
            .graphics_debugger
            .request_next_created_viewport_capture();
    }
}
