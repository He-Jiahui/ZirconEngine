use zircon_ui::UiFrame;

use super::workbench_activity_rail_pointer_layout::WorkbenchActivityRailPointerLayout;

pub(super) fn root_frame(layout: &WorkbenchActivityRailPointerLayout) -> UiFrame {
    let max_x = [layout.left_strip_frame, layout.right_strip_frame]
        .into_iter()
        .map(|frame| frame.x + frame.width)
        .fold(1.0_f32, f32::max);
    let max_y = [layout.left_strip_frame, layout.right_strip_frame]
        .into_iter()
        .map(|frame| frame.y + frame.height)
        .fold(1.0_f32, f32::max);
    UiFrame::new(0.0, 0.0, max_x.max(1.0), max_y.max(1.0))
}
