use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostMenuStateData, HostWindowPresentationData,
};

use super::super::super::super::frames::{
    constrained_menu_popup_frame, scrolled_menu_frame_with_state,
};
use super::source::RootMenuPopupSource;
use crate::ui::retained_host::menu_popup_contract::root_menu_popup_viewport;

pub(super) fn root_menu_popup_frame_with_state(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    source: &RootMenuPopupSource,
) -> FrameRect {
    let menu_frame_rect = scrolled_menu_frame_with_state(&source.menu_frame, menu_state);
    let viewport = root_menu_popup_viewport(
        source.menu_index,
        source.popup_height_px.max(1.0),
        menu_state.window_menu_popup_height_px,
        menu_state.window_menu_scroll_px,
    );
    constrained_menu_popup_frame(
        presentation,
        &menu_frame_rect,
        source.popup_width_px.max(menu_frame_rect.width).max(1.0),
        viewport.height,
    )
}
