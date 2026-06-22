use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::super::super::frames::{constrained_menu_popup_frame, scrolled_menu_frame};
use super::source::RootMenuPopupSource;

pub(super) fn root_menu_popup_frame(
    presentation: &HostWindowPresentationData,
    source: &RootMenuPopupSource,
) -> FrameRect {
    let menu_frame_rect = scrolled_menu_frame(&source.menu_frame, presentation);
    constrained_menu_popup_frame(
        presentation,
        &menu_frame_rect,
        source.popup_width_px.max(menu_frame_rect.width).max(1.0),
        source.popup_height_px.max(1.0),
    )
}
