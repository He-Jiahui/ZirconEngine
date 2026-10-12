use super::super::super::super::data::{
    paint_dock_overflow_menu_state, FrameRect, HostDockOverflowMenuStateData,
    HostWindowPresentationData,
};
use super::super::super::super::host_dock_overflow_menu::{
    host_dock_overflow_content_viewport_frame, host_dock_overflow_hidden_indices,
    host_dock_overflow_popup_frame_with_state, host_dock_overflow_projection,
    host_dock_overflow_row_frame_with_state, host_dock_overflow_scroll_content_extent,
    host_dock_overflow_scrollbar_reserve, host_dock_overflow_visible_row_range_with_state,
};
use super::super::super::super::menu_popup_metrics::MENU_POPUP_TEXT_INSET_X;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::{intersect, is_visible_frame};
use super::super::super::super::paint_primitives::{
    draw_rect_clipped, draw_rounded_box_clipped, draw_rounded_rect_clipped,
};
use super::super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::super::paint_theme::{current_host_metrics, current_host_palette};
use super::super::super::native_panes::draw_vertical_scrollbar;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in super::super) fn draw_host_dock_overflow_menu(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let state = paint_dock_overflow_menu_state(presentation);
    let Some(popup) = host_dock_overflow_popup_frame_with_state(presentation, &state) else {
        return;
    };
    if !is_visible_frame(&popup)
        || frame
            .paint_clip()
            .is_some_and(|damage| intersect(&popup, damage).is_none())
    {
        return;
    }
    let Some(projection) = host_dock_overflow_projection(presentation, &state) else {
        return;
    };
    let hidden_indices = host_dock_overflow_hidden_indices(&projection);
    let metrics = current_host_metrics();
    let palette = current_host_palette();
    draw_rounded_box_clipped(
        frame,
        popup.clone(),
        Some(&popup),
        palette.popup,
        palette.border,
        metrics.border_width,
        metrics.radius_panel,
    );
    let viewport = host_dock_overflow_content_viewport_frame(&popup);
    let scrollbar_reserve =
        host_dock_overflow_scrollbar_reserve(presentation, popup.height, &state);
    for row in host_dock_overflow_visible_row_range_with_state(presentation, &popup, &state) {
        let Some(tab_index) = hidden_indices.get(row).copied() else {
            continue;
        };
        let Some(tab) = projection.tabs.get(tab_index) else {
            continue;
        };
        let row_frame = host_dock_overflow_row_frame_with_state(presentation, &popup, row, &state);
        let hovered = state.hovered_tab_index == tab_index as i32;
        if tab.active || hovered {
            draw_rounded_rect_clipped(
                frame,
                row_frame.clone(),
                Some(&viewport),
                palette.surface_hover,
                metrics.radius_small,
            );
        }
        let selection_reserve =
            (metrics.selection_indicator_width + metrics.gap_s).min(row_frame.width.max(0.0));
        if tab.active {
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
        let title_frame = title_frame(&row_frame, selection_reserve, scrollbar_reserve);
        if is_visible_frame(&title_frame) {
            draw_text_with_size_and_style(
                frame,
                title_frame,
                tab.title.as_str(),
                Some(&viewport),
                if tab.active {
                    palette.text
                } else {
                    palette.text_muted
                },
                metrics.font_body,
                metrics.line_height(metrics.font_body),
                UiTextRunPaintStyle::default(),
            );
        }
    }
    draw_vertical_scrollbar(
        frame,
        &viewport,
        &popup,
        state.scroll_offset,
        host_dock_overflow_scroll_content_extent(presentation, &state),
        state.hovered_tab_index >= 0,
    );
}

fn title_frame(row: &FrameRect, selection_reserve: f32, scrollbar_reserve: f32) -> FrameRect {
    let metrics = current_host_metrics();
    let leading = (MENU_POPUP_TEXT_INSET_X + selection_reserve).min(row.width.max(0.0));
    let trailing =
        (MENU_POPUP_TEXT_INSET_X + scrollbar_reserve).min((row.width - leading).max(0.0));
    let line_height = metrics
        .line_height(metrics.font_body)
        .min(row.height.max(0.0));
    FrameRect {
        x: row.x + leading,
        y: row.y + (row.height - line_height).max(0.0) * 0.5,
        width: (row.width - leading - trailing).max(0.0),
        height: line_height,
    }
}
