use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, HostMenuChromeItemData, HostWindowPresentationData};
use super::super::super::menu_popup_metrics::{
    MENU_POPUP_SHORTCUT_RESERVED_WIDTH, MENU_POPUP_TEXT_INSET_X, MENU_POPUP_TEXT_INSET_Y,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{draw_rect_clipped, draw_text_bars_clipped};
use super::super::super::paint_theme::PALETTE;
use super::geometry::menu_popup_row_frame;

pub(in crate::ui::retained_host::host_contract) fn draw_menu_popup_rows(
    frame: &mut HostRgbaFrame,
    items: &ModelRc<HostMenuChromeItemData>,
    popup: &FrameRect,
    level: usize,
    presentation: &HostWindowPresentationData,
) {
    for row in 0..items.row_count() {
        let Some(item) = items.row_data(row) else {
            continue;
        };
        let row_frame = menu_popup_row_frame(popup, row, 0.0);
        let hovered = presentation
            .menu_state
            .hovered_menu_item_path
            .get(level)
            .is_some_and(|hovered_row| *hovered_row == row);
        if hovered {
            draw_rect_clipped(frame, row_frame.clone(), Some(popup), PALETTE.surface_hover);
        }
        let text_color = if item.enabled {
            PALETTE.text
        } else {
            PALETTE.text_disabled
        };
        draw_text_bars_clipped(
            frame,
            row_frame.x + MENU_POPUP_TEXT_INSET_X,
            row_frame.y + MENU_POPUP_TEXT_INSET_Y,
            item.label.as_str(),
            Some(popup),
            text_color,
        );
        if !item.shortcut.is_empty() {
            draw_text_bars_clipped(
                frame,
                (row_frame.x + row_frame.width - MENU_POPUP_SHORTCUT_RESERVED_WIDTH)
                    .max(row_frame.x + MENU_POPUP_TEXT_INSET_X),
                row_frame.y + MENU_POPUP_TEXT_INSET_Y,
                item.shortcut.as_str(),
                Some(popup),
                text_color,
            );
        }
    }
}
