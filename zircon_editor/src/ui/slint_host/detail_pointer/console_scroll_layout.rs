use zircon_ui::UiSize;

use super::console_constants::CONSOLE_VIEWPORT_Y;
use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;

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
