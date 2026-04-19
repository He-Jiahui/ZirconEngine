use zircon_runtime::ui::layout::UiSize;

use super::inspector_constants::INSPECTOR_VIEWPORT_Y;
use super::inspector_content_extent::inspector_content_extent;
use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;

pub(crate) fn inspector_scroll_layout(pane_size: UiSize) -> ScrollSurfacePointerLayout {
    ScrollSurfacePointerLayout {
        pane_size,
        viewport_origin_y: INSPECTOR_VIEWPORT_Y,
        content_extent: inspector_content_extent(),
    }
}
