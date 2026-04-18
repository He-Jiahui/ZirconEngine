use zircon_ui::UiFrame;

use super::workbench_drawer_header_pointer_layout::WorkbenchDrawerHeaderPointerLayout;

pub(super) fn root_frame(layout: &WorkbenchDrawerHeaderPointerLayout) -> UiFrame {
    let max_x = layout
        .surfaces
        .iter()
        .map(|surface| surface.strip_frame.x + surface.strip_frame.width)
        .fold(1.0_f32, f32::max);
    let max_y = layout
        .surfaces
        .iter()
        .map(|surface| surface.strip_frame.y + surface.strip_frame.height)
        .fold(1.0_f32, f32::max);
    UiFrame::new(0.0, 0.0, max_x.max(1.0), max_y.max(1.0))
}
