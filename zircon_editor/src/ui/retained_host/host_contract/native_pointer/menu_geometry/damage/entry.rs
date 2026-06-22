use super::super::super::super::data::{FrameRect, HostMenuStateData, HostWindowPresentationData};
use super::super::frames::shell_content_width;
use super::popup_bottom::open_menu_popup_bottom;

pub(in crate::ui::retained_host::host_contract) fn menu_damage_frame(
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    menu_damage_frame_with_state(presentation, &presentation.menu_state)
}

pub(in crate::ui::retained_host::host_contract) fn menu_damage_frame_with_state(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
) -> FrameRect {
    let scene = &presentation.host_scene_data;
    let width = shell_content_width(presentation);
    let base_height = scene.menu_chrome.top_bar_height_px.max(0.0);
    let popup_bottom = open_menu_popup_bottom(presentation, menu_state, base_height);
    FrameRect {
        x: 0.0,
        y: 0.0,
        width,
        height: (popup_bottom + 4.0).max(base_height),
    }
}
