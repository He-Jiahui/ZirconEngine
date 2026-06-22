use zircon_runtime_interface::ui::layout::UiFrame;

use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_from_ui(
    frame: UiFrame,
) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
