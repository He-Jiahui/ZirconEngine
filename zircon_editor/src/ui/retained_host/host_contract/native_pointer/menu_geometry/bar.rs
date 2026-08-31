use super::super::super::data::{HostMenuStateData, HostWindowPresentationData};
use super::super::routing::contains;
use super::frames::{menu_chrome_frame, scrolled_menu_frame_with_state, top_bar_fallback_frame};

pub(in crate::ui::retained_host::host_contract) fn menu_handles_point_with_state(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    x: f32,
    y: f32,
) -> bool {
    let scene = &presentation.host_scene_data;
    if contains(&menu_chrome_frame(scene), x, y) {
        return true;
    }
    if scene.menu_chrome.menu_frames.row_count() == 0 {
        return contains(&top_bar_fallback_frame(presentation), x, y);
    }
    scene.menu_chrome.menu_frames.iter().any(|control| {
        contains(
            &scrolled_menu_frame_with_state(&control.frame, menu_state),
            x,
            y,
        )
    })
}
