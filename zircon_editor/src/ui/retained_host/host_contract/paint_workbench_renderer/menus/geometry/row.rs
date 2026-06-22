use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn menu_popup_row_frame(
    popup: &FrameRect,
    row: usize,
    scroll_px: f32,
) -> FrameRect {
    FrameRect {
        x: popup.x + 6.0,
        y: popup.y + 6.0 + row as f32 * 30.0 - scroll_px,
        width: (popup.width - 12.0).max(0.0),
        height: 28.0,
    }
}
