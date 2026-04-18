use zircon_ui::UiSize;

use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;

impl Default for ScrollSurfacePointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            viewport_origin_y: 0.0,
            content_extent: 0.0,
        }
    }
}
