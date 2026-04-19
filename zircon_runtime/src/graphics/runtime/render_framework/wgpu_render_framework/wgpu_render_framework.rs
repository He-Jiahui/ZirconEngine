use std::sync::Mutex;

use super::super::render_framework_state::RenderFrameworkState;

pub struct WgpuRenderFramework {
    pub(in crate::graphics::runtime::render_framework) state: Mutex<RenderFrameworkState>,
}
