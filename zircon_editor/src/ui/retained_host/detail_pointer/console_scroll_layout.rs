use zircon_runtime_interface::ui::layout::UiSize;

use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;

const CONSOLE_VIEWPORT_Y: f32 = 0.0;

pub(crate) fn console_scroll_layout(
    pane_size: UiSize,
    content_extent: f32,
) -> ScrollSurfacePointerLayout {
    ScrollSurfacePointerLayout {
        pane_size,
        viewport_origin_y: CONSOLE_VIEWPORT_Y,
        content_extent,
    }
}
