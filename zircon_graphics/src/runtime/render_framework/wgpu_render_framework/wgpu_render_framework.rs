use std::sync::Mutex;

use super::super::render_framework_state::RenderFrameworkState;

pub struct WgpuRenderFramework {
    pub(in crate::runtime::render_framework) state: Mutex<RenderFrameworkState>,
}
