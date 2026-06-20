use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
use crate::core::framework::render::{RenderCapabilitySummary, RenderFrameworkError};
use crate::core::TaskPool;
#[cfg(test)]
use crate::core::{math::UVec2, resource::ResourceId};

#[cfg(test)]
use super::super::render_framework_backend_error::render_framework_backend_error;
use super::super::render_framework_state::RenderFrameworkState;

pub struct WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) state: Mutex<RenderFrameworkState>,
    pub(in crate::graphics::runtime::render_framework) operation_lock: Mutex<()>,
    pub(in crate::graphics::runtime::render_framework) compute_task_pool: TaskPool,
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

    #[cfg(test)]
    pub(crate) fn read_output_target_texture_rgba_for_tests(
        &self,
        texture_id: ResourceId,
    ) -> Result<Option<(UVec2, Vec<u8>)>, RenderFrameworkError> {
        let _operation_guard = self.lock_operation();
        let state = self.lock_state();
        state
            .renderer
            .read_output_target_texture_rgba_for_tests(&texture_id)
            .map_err(render_framework_backend_error)
    }
}
