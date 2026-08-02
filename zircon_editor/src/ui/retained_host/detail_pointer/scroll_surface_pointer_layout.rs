use zircon_runtime_interface::ui::layout::UiSize;

use super::viewport_frame::viewport_frame;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScrollSurfacePointerLayout {
    pub(super) pane_size: UiSize,
    pub(super) viewport_origin_y: f32,
    pub(super) content_extent: f32,
}

impl ScrollSurfacePointerLayout {
    pub(crate) fn max_scroll_offset(&self) -> f32 {
        (self.content_extent - viewport_frame(self).height).max(0.0)
    }
}
