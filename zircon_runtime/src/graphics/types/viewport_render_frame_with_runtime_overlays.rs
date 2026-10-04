use crate::core::framework::render::RenderOverlayExtract;

use super::viewport_render_frame::ViewportRenderFrame;

// 调试覆盖层是可选替代值；缺省时仍使用 extract 中的原始覆盖层。
impl ViewportRenderFrame {
    pub(crate) fn with_runtime_overlays(mut self, overlays: RenderOverlayExtract) -> Self {
        self.runtime_overlay_override = Some(overlays);
        self
    }
}
