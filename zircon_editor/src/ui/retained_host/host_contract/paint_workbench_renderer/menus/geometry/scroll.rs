use super::super::super::super::data::{paint_menu_state, FrameRect, HostWindowPresentationData};

pub(in crate::ui::retained_host::host_contract) fn scrolled_menu_frame(
    menu_frame: &FrameRect,
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    let menu_state = paint_menu_state(presentation);
    FrameRect {
        x: menu_frame.x - menu_state.menu_bar_scroll_px,
        y: menu_frame.y,
        width: menu_frame.width,
        height: menu_frame.height,
    }
}
