mod submenus;

use super::super::super::data::HostWindowPresentationData;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::{
    draw_rounded_border_clipped, draw_rounded_rect_clipped,
};
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
    let viewport = root_menu_popup_viewport(
        menu_index,
        menu.popup_height_px.max(1.0),
        presentation.menu_state.window_menu_popup_height_px,
        presentation.menu_state.window_menu_scroll_px,
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
    draw_rounded_rect_clipped(
        frame,
        popup.clone(),
        Some(&popup),
        palette.surface,
        metrics.radius_control,
    );
    draw_rounded_border_clipped(
        frame,
        popup.clone(),
        Some(&popup),
        palette.border,
        metrics.border_width,
        metrics.radius_control,
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
    draw_open_submenu_popups(frame, presentation, menu.items.clone(), popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn menu_popup_palette_uses_the_popup_and_border_theme_roles() {
        let mut palette = PALETTE;
        palette.popup = [18, 26, 35, 255];
        palette.border = [67, 89, 101, 255];

        let projected = menu_popup_palette(palette);

        assert_eq!(projected.surface, palette.popup);
        assert_eq!(projected.border, palette.border);
    }
}
