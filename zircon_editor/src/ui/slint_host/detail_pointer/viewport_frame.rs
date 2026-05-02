use zircon_runtime_interface::ui::layout::UiFrame;

use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;

pub(super) fn viewport_frame(layout: &ScrollSurfacePointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        layout.viewport_origin_y.max(0.0),
        layout.pane_size.width.max(0.0),
        (layout.pane_size.height - layout.viewport_origin_y).max(0.0),
    )
}
