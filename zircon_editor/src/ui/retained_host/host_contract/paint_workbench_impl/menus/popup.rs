use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, HostMenuChromeItemData, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::{draw_border, draw_rect};
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::{ACCENT, TOP_BAR};
use super::geometry::{
    constrained_menu_popup_frame, constrained_submenu_popup_frame, menu_popup_height,
    menu_popup_row_frame, scrolled_menu_frame,
};
use super::rows::draw_menu_popup_rows;

pub(super) fn draw_open_menu_popup(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let menu_index = presentation.menu_state.open_menu_index;
    if menu_index < 0 {
        return;
    }
    let menu_index = menu_index as usize;
    let scene = &presentation.host_scene_data;
    let Some(menu_frame) = scene.menu_chrome.menu_frames.row_data(menu_index) else {
        return;
    };
    let Some(menu) = scene.menu_chrome.menus.row_data(menu_index) else {
        return;
    };
    let menu_frame_rect = scrolled_menu_frame(&menu_frame.frame, presentation);
    let popup = constrained_menu_popup_frame(
        presentation,
        &menu_frame_rect,
        menu.popup_width_px.max(menu_frame_rect.width).max(1.0),
        menu.popup_height_px.max(1.0),
    );
    if !is_visible_frame(&popup) {
        return;
    }
    draw_rect(frame, popup.clone(), TOP_BAR);
    draw_border(frame, popup.clone(), ACCENT);
    if menu.popup_nodes.row_count() > 0 {
        draw_template_nodes(frame, &menu.popup_nodes, &popup, &popup, None);
    } else {
        draw_menu_popup_rows(frame, &menu.items, &popup, 0, presentation);
    }
    draw_open_submenu_popups(frame, presentation, menu.items.clone(), popup);
}

fn draw_open_submenu_popups(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    mut items: ModelRc<HostMenuChromeItemData>,
    mut parent_popup: FrameRect,
) {
    for (level, selected_index) in presentation
        .menu_state
        .open_submenu_path
        .iter()
        .copied()
        .enumerate()
    {
        let Some(branch) = items.row_data(selected_index) else {
            break;
        };
        if branch.children.row_count() == 0 {
            break;
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
        if !is_visible_frame(&popup) {
            break;
        }
        draw_rect(frame, popup.clone(), TOP_BAR);
        draw_border(frame, popup.clone(), ACCENT);
        draw_menu_popup_rows(frame, &branch.children, &popup, level + 1, presentation);

        items = branch.children.clone();
        parent_popup = popup;
    }
}
