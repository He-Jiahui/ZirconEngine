use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn scrolled_menu_frame(
    menu_frame: &FrameRect,
    menu_bar_scroll_px: f32,
) -> FrameRect {
    FrameRect {
        x: menu_frame.x - menu_bar_scroll_px,
        y: menu_frame.y,
        width: menu_frame.width,
        height: menu_frame.height,
    }
}
