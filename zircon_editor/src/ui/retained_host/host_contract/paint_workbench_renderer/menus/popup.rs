mod submenus;

use super::super::super::data::HostWindowPresentationData;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::{draw_border, draw_rect};
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::{ACCENT, TOP_BAR};
use super::geometry::{constrained_menu_popup_frame, scrolled_menu_frame};
use super::rows::draw_menu_popup_rows;
use submenus::draw_open_submenu_popups;

pub(in crate::ui::retained_host::host_contract) fn draw_open_menu_popup(
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
