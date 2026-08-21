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
use super::super::super::super::paint_geometry::is_visible_frame;
use super::super::super::super::paint_primitives::{
    draw_rect_clipped, draw_rounded_border_clipped, draw_rounded_rect_clipped,
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

    let metrics = current_host_metrics();
    let palette = page_overflow_palette(current_host_palette());
    draw_rounded_rect_clipped(
        frame,
        popup.clone(),
        Some(&popup),
        palette.popup,
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
            .row_data(page_index)
        else {
            continue;
        };
        let row_frame = host_page_overflow_row_frame_with_state(presentation, &popup, row, &state);
        let active = tab.active;
        let hovered = is_hovered(&state, page_index);
        if active || hovered {
            draw_rect_clipped(frame, row_frame.clone(), Some(&viewport), palette.hover);
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
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::ui::retained_host::host_contract::data::TabData;
    use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
    use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};

    #[test]
    fn overflow_palette_projects_each_runtime_theme_role() {
        let mut palette = PALETTE;
        palette.popup = [1, 2, 3, 4];
        palette.border = [5, 6, 7, 8];
        palette.surface_hover = [9, 10, 11, 12];
        palette.accent = [13, 14, 15, 16];
        palette.text = [17, 18, 19, 20];
        palette.text_muted = [21, 22, 23, 24];

        assert_eq!(
            page_overflow_palette(palette),
            PageOverflowPalette {
                popup: [1, 2, 3, 4],
                border: [5, 6, 7, 8],
                hover: [9, 10, 11, 12],
                accent: [13, 14, 15, 16],
                text: [17, 18, 19, 20],
                text_muted: [21, 22, 23, 24],
            }
        );
    }

    #[test]
    fn overflowing_row_title_reserves_the_scrollbar_and_its_gap() {
        let row = FrameRect {
            x: 100.0,
            y: 40.0,
            width: 100.0,
            height: 28.0,
        };
        let scrollbar_reserve = 12.0;

        let title = overflow_row_title_frame(&row, 0.0, scrollbar_reserve);

        assert_eq!(
            title.x + title.width,
            row.x + row.width - MENU_POPUP_TEXT_INSET_X - scrollbar_reserve
        );
    }

    #[test]
    fn overflow_row_title_uses_a_finite_runtime_text_slot_with_ellipsis() {
        let presentation =
            overflow_presentation("A long hidden editor tab title that must ellipsize");
        let mut frame = HostRgbaFrame::recording_only(240, 180);

        draw_host_page_overflow_menu(&mut frame, &presentation);

        let popup = host_page_overflow_popup_frame(&presentation)
            .expect("open overflow should provide a popup frame");
        let row = host_page_overflow_row_frame(&presentation, &popup, 0);
        let command = frame
            .into_recorded_commands()
            .into_iter()
            .find(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
            .expect("overflow row title should use Runtime Text");
        let HostRecordedPaintKind::Text { text, .. } = &command.kind else {
            unreachable!("filtered command should be text");
        };

        assert!(text.ends_with('\u{2026}'));
        assert!(command.frame.x >= row.x + MENU_POPUP_TEXT_INSET_X);
        assert!(
            command.frame.x + command.frame.width <= row.x + row.width - MENU_POPUP_TEXT_INSET_X
        );
    }

    fn overflow_presentation(title: &str) -> HostWindowPresentationData {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_layout.status_bar_frame = FrameRect {
            x: 0.0,
            y: 160.0,
            width: 240.0,
            height: 20.0,
        };
        presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
            x: 188.0,
            y: 24.0,
            width: 34.0,
            height: 28.0,
        };
        presentation.host_scene_data.page_chrome.tabs = model_rc(vec![TabData {
            id: "long-tab".into(),
            title: title.into(),
            ..TabData::default()
        }]);
        let metrics = current_host_metrics();
        presentation
            .host_scene_data
            .page_chrome
            .overflow_widest_title_width_px =
            measure_runtime_text_width(title, metrics.font_body) + metrics.text_clip_guard;
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = vec![0];
        presentation.host_page_overflow_menu_state = HostPageOverflowMenuStateData {
            open: true,
            hovered_page_index: -1,
            scroll_offset: 0.0,
        };
        presentation
    }

    fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
        ModelRc::from(Rc::new(VecModel::from(rows)))
    }
}
