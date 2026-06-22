use super::super::super::super::super::data::{HostMenuStateData, HostWindowPresentationData};
use super::super::super::frames::{constrained_menu_popup_frame, scrolled_menu_frame};
use super::super::stack::menu_popup_stack_bottom;

pub(super) fn opened_root_menu_popup_bottom(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
) -> Option<f32> {
    let scene = &presentation.host_scene_data;
    let menu = scene
        .menu_chrome
        .menus
        .row_data(menu_state.open_menu_index as usize)?;
    let menu_frame = scene
        .menu_chrome
        .menu_frames
        .row_data(menu_state.open_menu_index as usize)?;
    let menu_frame_rect = scrolled_menu_frame(&menu_frame.frame, presentation);
    let popup = constrained_menu_popup_frame(
        presentation,
        &menu_frame_rect,
        menu.popup_width_px.max(menu_frame_rect.width).max(1.0),
        menu.popup_height_px.max(1.0),
    );
    Some(menu_popup_stack_bottom(
        presentation,
        menu_state,
        menu.items.clone(),
        popup,
    ))
}
