use crate::core::framework::render::RenderOverlayExtract;

use super::viewport_render_frame::ViewportRenderFrame;

impl ViewportRenderFrame {
    pub(crate) fn with_runtime_overlays(mut self, overlays: RenderOverlayExtract) -> Self {
        self.runtime_overlay_override = Some(overlays);
        self
    }
}
