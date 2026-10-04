mod submenus;

use super::super::super::data::{paint_menu_state, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::draw_rounded_box_clipped;
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostMaterialPalette,
};
use super::super::native_panes::draw_vertical_scrollbar;
use super::geometry::{constrained_menu_popup_frame, scrolled_menu_frame};
use super::rows::draw_menu_popup_rows;
use crate::ui::retained_host::menu_popup_contract::root_menu_popup_viewport;
use submenus::draw_open_submenu_popups;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MenuPopupPalette {
    pub(super) surface: [u8; 4],
    pub(super) border: [u8; 4],
}

pub(super) fn menu_popup_palette(palette: HostMaterialPalette) -> MenuPopupPalette {
    MenuPopupPalette {
        surface: palette.popup,
        border: palette.border,
    }
}

// 根菜单只消费当前打开项的投影状态；可见行、滚动条和后续子菜单在同一弹层几何下绘制。
pub(in crate::ui::retained_host::host_contract) fn draw_open_menu_popup(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let menu_state = paint_menu_state(presentation);
    let menu_index = menu_state.open_menu_index;
    if menu_index < 0 {
        return;
    }
    let menu_index = menu_index as usize;
    let scene = &presentation.host_scene_data;
    let Some(menu_frame) = scene.menu_chrome.menu_frames.get(menu_index) else {
        return;
    };
    let Some(menu) = scene.menu_chrome.menus.get(menu_index) else {
        return;
    };
    let menu_frame_rect = scrolled_menu_frame(&menu_frame.frame, menu_state.menu_bar_scroll_px);
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
    if !is_visible_frame(&popup) {
        return;
    }
    let metrics = current_host_metrics();
    let palette = menu_popup_palette(current_host_palette());
    draw_rounded_box_clipped(
        frame,
        popup.clone(),
        Some(&popup),
        palette.surface,
        palette.border,
        metrics.border_width,
        metrics.radius_panel,
    );
    if menu.popup_nodes.row_count() > 0 {
        draw_template_nodes(frame, &menu.popup_nodes, &popup, &popup, None);
    } else {
        draw_menu_popup_rows(frame, &menu.items, &popup, 0, viewport.scroll, presentation);
    }
    draw_vertical_scrollbar(
        frame,
        &popup,
        &popup,
        viewport.scroll,
        menu.popup_height_px,
        false,
    );
    draw_open_submenu_popups(frame, presentation, &menu_state, menu.items.clone(), popup);
}

#[cfg(test)]
#[path = "tests/popup.rs"]
mod tests;
