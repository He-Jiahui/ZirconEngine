use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use crate::graphics::ViewportRenderFrame;

use super::submit_frame_extract::submit_runtime_frame as submit_runtime_frame_impl;
use super::wgpu_render_framework::WgpuRenderFramework;

impl WgpuRenderFramework {
    pub fn submit_runtime_frame(
        &self,
        viewport: RenderViewportHandle,
        frame: ViewportRenderFrame,
    ) -> Result<(), RenderFrameworkError> {
        submit_runtime_frame_impl(self, viewport, frame)
    }
}
