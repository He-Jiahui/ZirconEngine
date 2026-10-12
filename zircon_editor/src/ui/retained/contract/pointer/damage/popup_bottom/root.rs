use super::super::super::super::super::data::{HostMenuStateData, HostWindowPresentationData};
use super::super::super::frames::{constrained_menu_popup_frame, scrolled_menu_frame_with_state};
use super::super::stack::menu_popup_stack_bottom;
use crate::ui::retained_host::menu_popup_contract::root_menu_popup_viewport;

pub(super) fn opened_root_menu_popup_bottom(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
) -> Option<f32> {
    let scene = &presentation.host_scene_data;
    let menu = scene
        .menu_chrome
        .menus
        .get(menu_state.open_menu_index as usize)?;
    let menu_frame = scene
        .menu_chrome
        .menu_frames
        .get(menu_state.open_menu_index as usize)?;
    let menu_frame_rect = scrolled_menu_frame_with_state(&menu_frame.frame, menu_state);
    let menu_index = menu_state.open_menu_index as usize;
    let viewport = root_menu_popup_viewport(
        menu_index,
        menu.popup_height_px.max(1.0),
        menu_state.window_menu_popup_height_px,
        menu_state.window_menu_scroll_px,
    );
    let popup = constrained_menu_popup_frame(
        presentation,
        &menu_frame_rect,
        menu.popup_width_px.max(menu_frame_rect.width).max(1.0),
        viewport.height,
    );
    Some(menu_popup_stack_bottom(
        presentation,
        menu_state,
        menu.items.clone(),
        popup,
    ))
}
