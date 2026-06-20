use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, HostMenuChromeItemData, HostWindowPresentationData};
use super::super::routing::contains;
use super::frames::{
    constrained_menu_popup_frame, constrained_submenu_popup_frame, menu_popup_height,
    menu_popup_row_frame, popup_blocking_frame, scrolled_menu_frame,
};

pub(in crate::ui::retained_host::host_contract) fn menu_popup_handles_point(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> bool {
    let state = &presentation.menu_state;
    if state.open_menu_index < 0 {
        return false;
    }
    let menu_index = state.open_menu_index as usize;
    let Some(menu_frame) = presentation
        .host_scene_data
        .menu_chrome
        .menu_frames
        .row_data(menu_index)
    else {
        return false;
    };
    let Some(menu) = presentation
        .host_scene_data
        .menu_chrome
        .menus
        .row_data(menu_index)
    else {
        return false;
    };
    let menu_frame_rect = scrolled_menu_frame(&menu_frame.frame, presentation);
    let popup = constrained_menu_popup_frame(
        presentation,
        &menu_frame_rect,
        menu.popup_width_px.max(menu_frame_rect.width).max(1.0),
        menu.popup_height_px.max(1.0),
    );
    contains(&popup, x, y)
        || nested_menu_popup_handles_point(presentation, menu.items.clone(), popup, x, y)
        || contains(&popup_blocking_frame(presentation), x, y)
}

fn nested_menu_popup_handles_point(
    presentation: &HostWindowPresentationData,
    mut items: ModelRc<HostMenuChromeItemData>,
    mut parent_popup: FrameRect,
    x: f32,
    y: f32,
) -> bool {
    for (level, selected_index) in presentation
        .menu_state
        .open_submenu_path
        .iter()
        .copied()
        .enumerate()
    {
        let Some(branch) = items.row_data(selected_index) else {
            return false;
        };
        if branch.children.row_count() == 0 {
            return false;
        }
        let scroll_px = if level == 0 {
            presentation.menu_state.window_menu_scroll_px
        } else {
            0.0
        };
        let anchor = menu_popup_row_frame(&parent_popup, selected_index, scroll_px);
        let popup = constrained_submenu_popup_frame(
            presentation,
            &anchor,
            parent_popup.width.max(1.0),
            menu_popup_height(branch.children.row_count()).max(1.0),
        );
        if contains(&popup, x, y) {
            return true;
        }
        items = branch.children.clone();
        parent_popup = popup;
    }
    false
}
