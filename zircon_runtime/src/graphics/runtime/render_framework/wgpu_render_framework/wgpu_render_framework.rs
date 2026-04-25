use std::sync::Mutex;

#[cfg(test)]
use crate::core::framework::render::RenderCapabilitySummary;

use super::super::render_framework_state::RenderFrameworkState;

pub struct WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) state: Mutex<RenderFrameworkState>,
}

impl WgpuRenderFramework {
    #[cfg(test)]
    pub(crate) fn override_capabilities_for_tests(&self, capabilities: RenderCapabilitySummary) {
        self.state.lock().unwrap().stats.capabilities = capabilities;
    }
}
