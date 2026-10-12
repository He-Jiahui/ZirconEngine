use super::super::super::super::data::{
    paint_page_overflow_menu_state, FrameRect, HostPageOverflowMenuStateData,
    HostWindowPresentationData,
};
use super::super::super::super::host_page_overflow_menu::{
    host_page_overflow_content_viewport_frame, host_page_overflow_popup_frame,
    host_page_overflow_popup_frame_with_state, host_page_overflow_row_frame,
    host_page_overflow_row_frame_with_state, host_page_overflow_scroll_content_extent,
    host_page_overflow_scrollbar_reserve, host_page_overflow_visible_row_range_with_state,
};
use super::super::super::super::menu_popup_metrics::MENU_POPUP_TEXT_INSET_X;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::{intersect, is_visible_frame};
use super::super::super::super::paint_primitives::{
    draw_rect_clipped, draw_rounded_box_clipped, draw_rounded_rect_clipped,
};
use super::super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostMaterialPalette,
};
use super::super::super::native_panes::draw_vertical_scrollbar;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PageOverflowPalette {
    popup: [u8; 4],
    border: [u8; 4],
    hover: [u8; 4],
    accent: [u8; 4],
    text: [u8; 4],
    text_muted: [u8; 4],
}

pub(in super::super) fn draw_host_page_overflow_menu(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let state = paint_page_overflow_menu_state(presentation);
    let Some(popup) = host_page_overflow_popup_frame_with_state(presentation, &state) else {
        return;
    };
    if !is_visible_frame(&popup) {
        return;
    }
    if frame
        .paint_clip()
        .is_some_and(|damage| intersect(&popup, damage).is_none())
    {
        return;
    }

    let metrics = current_host_metrics();
    let palette = page_overflow_palette(current_host_palette());
    draw_rounded_box_clipped(
        frame,
        popup.clone(),
        Some(&popup),
        palette.popup,
        palette.border,
        metrics.border_width,
        metrics.radius_panel,
    );
    let viewport = host_page_overflow_content_viewport_frame(&popup);
    let scroll_content_extent = host_page_overflow_scroll_content_extent(presentation);
    let scrollbar_reserve = host_page_overflow_scrollbar_reserve(presentation, popup.height);

    for row in host_page_overflow_visible_row_range_with_state(presentation, &popup, &state) {
        let page_index = presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices[row];
        let Some(tab) = presentation
            .host_scene_data
            .page_chrome
            .tabs
            .get(page_index)
        else {
            continue;
        };
        let row_frame = host_page_overflow_row_frame_with_state(presentation, &popup, row, &state);
        let active = tab.active;
        let hovered = is_hovered(&state, page_index);
        if active || hovered {
            draw_rounded_rect_clipped(
                frame,
                row_frame.clone(),
                Some(&viewport),
                palette.hover,
                metrics.radius_small,
            );
        }
        let selection_reserve =
            (metrics.selection_indicator_width + metrics.gap_s).min(row_frame.width.max(0.0));
        if active {
            draw_rect_clipped(
                frame,
                FrameRect {
                    x: row_frame.x,
                    y: row_frame.y,
                    width: metrics
                        .selection_indicator_width
                        .min(row_frame.width.max(0.0)),
                    height: row_frame.height,
                },
                Some(&viewport),
                palette.accent,
            );
        }
        let title_frame =
            overflow_row_title_frame(&row_frame, selection_reserve, scrollbar_reserve);
        if !is_visible_frame(&title_frame) {
            continue;
        }
        draw_text_with_size_and_style(
            frame,
            title_frame,
            tab.title.as_str(),
            Some(&viewport),
            if active {
                palette.text
            } else {
                palette.text_muted
            },
            metrics.font_body,
            metrics.line_height(metrics.font_body),
            UiTextRunPaintStyle::default(),
        );
    }

    draw_vertical_scrollbar(
        frame,
        &viewport,
        &popup,
        state.scroll_offset,
        scroll_content_extent,
        state.hovered_page_index >= 0,
    );
}

fn page_overflow_palette(palette: HostMaterialPalette) -> PageOverflowPalette {
    PageOverflowPalette {
        popup: palette.popup,
        border: palette.border,
        hover: palette.surface_hover,
        accent: palette.accent,
        text: palette.text,
        text_muted: palette.text_muted,
    }
}

fn is_hovered(state: &HostPageOverflowMenuStateData, page_index: usize) -> bool {
    state.hovered_page_index >= 0 && state.hovered_page_index as usize == page_index
}

fn overflow_row_title_frame(
    row: &FrameRect,
    selection_reserve: f32,
    scrollbar_reserve: f32,
) -> FrameRect {
    let metrics = current_host_metrics();
    let leading_inset = (MENU_POPUP_TEXT_INSET_X + selection_reserve).min(row.width.max(0.0));
    let trailing_inset = (MENU_POPUP_TEXT_INSET_X + scrollbar_reserve.max(0.0))
        .min((row.width - leading_inset).max(0.0));
    let line_height = metrics
        .line_height(metrics.font_body)
        .min(row.height.max(0.0));
    FrameRect {
        x: row.x + leading_inset,
        y: row.y + ((row.height - line_height).max(0.0) * 0.5),
        width: (row.width - leading_inset - trailing_inset).max(0.0),
        height: line_height,
    }
}

#[cfg(test)]
#[path = "tests/page_overflow.rs"]
mod tests;
