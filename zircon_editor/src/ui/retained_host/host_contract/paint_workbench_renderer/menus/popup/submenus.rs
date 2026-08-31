use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{
    FrameRect, HostMenuChromeItemData, HostMenuStateData, HostWindowPresentationData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::is_visible_frame;
use super::super::super::super::paint_primitives::draw_rounded_box_clipped;
use super::super::super::super::paint_theme::{current_host_metrics, current_host_palette};
use super::super::geometry::{
    constrained_submenu_popup_frame, menu_popup_height, menu_popup_row_frame,
};
use super::super::rows::draw_menu_popup_rows;
use super::menu_popup_palette;

pub(super) fn draw_open_submenu_popups(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    mut items: ModelRc<HostMenuChromeItemData>,
    mut parent_popup: FrameRect,
) {
    let metrics = current_host_metrics();
    let palette = menu_popup_palette(current_host_palette());
    for (level, selected_index) in menu_state.open_submenu_path.iter().copied().enumerate() {
        let Some(branch) = items.get(selected_index) else {
            break;
        };
        if branch.children.row_count() == 0 {
            break;
        }

        let scroll_px = if level == 0 {
            menu_state.window_menu_scroll_px
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
        draw_rounded_box_clipped(
            frame,
            popup.clone(),
            Some(&popup),
            palette.surface,
            palette.border,
            metrics.border_width,
            metrics.radius_panel,
        );
        draw_menu_popup_rows(
            frame,
            &branch.children,
            &popup,
            level + 1,
            0.0,
            presentation,
        );

        items = branch.children.clone();
        parent_popup = popup;
    }
}
