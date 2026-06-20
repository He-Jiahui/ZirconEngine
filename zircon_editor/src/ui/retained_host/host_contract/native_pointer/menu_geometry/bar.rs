use super::super::super::data::HostWindowPresentationData;
use super::super::routing::contains;
use super::frames::{menu_chrome_frame, scrolled_menu_frame, top_bar_fallback_frame};

pub(in crate::ui::retained_host::host_contract) fn menu_handles_point(
    presentation: &HostWindowPresentationData,
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
    (0..scene.menu_chrome.menu_frames.row_count()).any(|row| {
        scene
            .menu_chrome
            .menu_frames
            .row_data(row)
            .is_some_and(|control| {
                contains(&scrolled_menu_frame(&control.frame, presentation), x, y)
            })
    })
}
