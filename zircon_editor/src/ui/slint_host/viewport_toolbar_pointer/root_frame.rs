use zircon_runtime::ui::layout::UiFrame;

use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;

pub(super) fn root_frame(layout: &ViewportToolbarPointerLayout) -> UiFrame {
    let max_x = layout
        .surfaces
        .iter()
        .map(|surface| surface.frame.x + surface.frame.width)
        .fold(1.0_f32, f32::max);
    let max_y = layout
        .surfaces
        .iter()
        .map(|surface| surface.frame.y + surface.frame.height)
        .fold(1.0_f32, f32::max);
    UiFrame::new(0.0, 0.0, max_x.max(1.0), max_y.max(1.0))
}
