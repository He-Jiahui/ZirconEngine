mod text_layout;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, HostMenuChromeItemData, HostWindowPresentationData};
use super::super::super::menu_popup_metrics::{
    menu_popup_visible_row_range, MENU_POPUP_EDGE_INSET, MENU_POPUP_TEXT_INSET_X,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::draw_rounded_rect_clipped;
use super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::paint_theme::{current_host_metrics, current_host_palette};
use super::geometry::menu_popup_row_frame;
use text_layout::menu_row_text_columns;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

// 根菜单与各级子菜单共用行绘制；可见范围、悬停路径和快捷键列都以当前层级及弹层框解释。
pub(in crate::ui::retained_host::host_contract) fn draw_menu_popup_rows(
    frame: &mut HostRgbaFrame,
    items: &ModelRc<HostMenuChromeItemData>,
    popup: &FrameRect,
    level: usize,
    scroll_px: f32,
    presentation: &HostWindowPresentationData,
) {
    let metrics = current_host_metrics();
    let palette = current_host_palette();
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());
    for row in menu_popup_visible_row_range(
        items.row_count(),
        popup.height,
        scroll_px,
        MENU_POPUP_EDGE_INSET,
    ) {
        let Some(item) = items.get(row) else {
            continue;
        };
        let row_frame = menu_popup_row_frame(popup, row, scroll_px);
        let hovered = presentation
            .menu_state
            .hovered_menu_item_path
            .get(level)
            .is_some_and(|hovered_row| *hovered_row == row);
        if hovered {
            draw_rounded_rect_clipped(
                frame,
                row_frame.clone(),
                Some(popup),
                palette.surface_hover,
                metrics.radius_small,
            );
        }
        let text_color = if item.enabled {
            palette.text
        } else {
            palette.text_disabled
        };
        let text_columns = menu_row_text_columns(&row_frame, popup, item.shortcut.as_str());
        let label_frame = menu_row_text_frame(
            &row_frame,
            row_frame.x + MENU_POPUP_TEXT_INSET_X,
            text_columns.label_clip.x + text_columns.label_clip.width,
            line_height,
        );
        draw_text_with_size_and_style(
            frame,
            label_frame,
            item.label.as_str(),
            Some(&text_columns.label_clip),
            text_color,
            metrics.font_body,
            line_height,
            UiTextRunPaintStyle::default(),
        );
        if let Some(shortcut_x) = text_columns.shortcut_x {
            let shortcut_frame = menu_row_text_frame(
                &row_frame,
                shortcut_x,
                row_frame.x + row_frame.width - MENU_POPUP_TEXT_INSET_X,
                line_height,
            );
            draw_text_with_size_and_style(
                frame,
                shortcut_frame,
                item.shortcut.as_str(),
                Some(popup),
                text_color,
                metrics.font_body,
                line_height,
                UiTextRunPaintStyle::default(),
            );
        }
    }
}

fn menu_row_text_frame(row: &FrameRect, x: f32, right: f32, line_height: f32) -> FrameRect {
    let line_height = line_height.min(row.height.max(1.0));
    FrameRect {
        x,
        y: row.y + ((row.height - line_height).max(0.0) * 0.5),
        width: (right - x).max(1.0),
        height: line_height,
    }
}

#[cfg(test)]
#[path = "tests/rows.rs"]
mod tests;
