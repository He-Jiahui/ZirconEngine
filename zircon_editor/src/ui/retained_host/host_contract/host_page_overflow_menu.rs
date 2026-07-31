use super::data::{FrameRect, HostWindowPresentationData};
use super::menu_popup_metrics::{
    menu_popup_outer_padding, menu_popup_row_stride, menu_popup_shell_padding,
    MENU_POPUP_ANCHOR_GAP, MENU_POPUP_EDGE_INSET, MENU_POPUP_ROW_HEIGHT, MENU_POPUP_SHELL_MARGIN,
    MENU_POPUP_TEXT_INSET_X,
};
#[cfg(test)]
use super::paint_text::measure_runtime_text_width;
use super::paint_theme::current_host_metrics;
use crate::ui::retained_host::popup_anchor_metrics::clamp_popup_x_to_bounds;
use crate::ui::workbench::page_tabs::MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH;

pub(in crate::ui::retained_host::host_contract) struct HostPageOverflowRowHit {
    pub page_index: usize,
    pub frame: FrameRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct HostPageOverflowVerticalPlacement {
    y: f32,
    height: f32,
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_popup_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    if !presentation.host_page_overflow_menu_state.open {
        return None;
    }
    let overflow = &presentation.host_scene_data.page_chrome.overflow_frame;
    let overflow_right = overflow.x + overflow.width;
    let overflow_bottom = overflow.y + overflow.height;
    if !overflow.x.is_finite()
        || !overflow.y.is_finite()
        || !overflow.width.is_finite()
        || !overflow.height.is_finite()
        || !overflow_right.is_finite()
        || !overflow_bottom.is_finite()
        || overflow.width <= 0.0
        || overflow.height <= 0.0
    {
        return None;
    }
    let item_count = presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices
        .len();
    if item_count == 0 {
        return None;
    }
    let content_height = host_page_overflow_content_extent(presentation);
    let shell_bottom = [
        presentation.host_layout.status_bar_frame.y,
        presentation.host_scene_data.layout.status_bar_frame.y,
    ]
    .into_iter()
    .filter(|value| value.is_finite() && *value > 0.0)
    .fold(0.0_f32, f32::max);
    let vertical = host_page_overflow_vertical_placement(overflow, shell_bottom, content_height);
    if !vertical.height.is_finite() || vertical.height <= 0.0 {
        return None;
    }
    let scrollbar_reserve = host_page_overflow_scrollbar_reserve(presentation, vertical.height);
    let (shell_x, shell_width) = host_page_overflow_shell_bounds(presentation, overflow);
    let popup_width = host_page_overflow_popup_width(presentation, shell_width, scrollbar_reserve);
    if !popup_width.is_finite() || popup_width <= 0.0 {
        return None;
    }
    let x = clamp_popup_x_to_bounds(
        overflow.x + overflow.width - popup_width,
        shell_x,
        shell_width,
        popup_width,
    );
    let popup = FrameRect {
        x,
        y: vertical.y,
        width: popup_width,
        height: vertical.height,
    };
    let viewport = host_page_overflow_content_viewport_frame(&popup);
    (viewport.width.is_finite()
        && viewport.width >= MENU_POPUP_EDGE_INSET
        && viewport.height.is_finite()
        && viewport.height >= MENU_POPUP_EDGE_INSET)
        .then_some(popup)
}

fn host_page_overflow_popup_width(
    presentation: &HostWindowPresentationData,
    shell_width: f32,
    scrollbar_reserve: f32,
) -> f32 {
    let metrics = current_host_metrics();
    let widest_title = presentation
        .host_scene_data
        .page_chrome
        .overflow_widest_title_width_px
        .max(0.0);
    let title_chrome = MENU_POPUP_EDGE_INSET * 2.0
        + MENU_POPUP_TEXT_INSET_X * 2.0
        + metrics.selection_indicator_width
        + metrics.gap_s
        + scrollbar_reserve.max(0.0);
    let preferred_width = (widest_title + title_chrome).max(MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH);

    preferred_width.min((shell_width - menu_popup_shell_padding()).max(0.0))
}

fn host_page_overflow_shell_bounds(
    presentation: &HostWindowPresentationData,
    overflow: &FrameRect,
) -> (f32, f32) {
    let mut shell_left = f32::INFINITY;
    let mut shell_right = f32::NEG_INFINITY;
    for frame in [
        &presentation.host_layout.status_bar_frame,
        &presentation.host_layout.center_band_frame,
    ] {
        let right = frame.x + frame.width;
        if frame.x.is_finite() && frame.width.is_finite() && frame.width > 0.0 && right.is_finite()
        {
            shell_left = shell_left.min(frame.x);
            shell_right = shell_right.max(right);
        }
    }
    if shell_left.is_finite() && shell_right.is_finite() && shell_right > shell_left {
        return (shell_left, (shell_right - shell_left).max(0.0));
    }

    let right = (overflow.x + overflow.width).max(0.0);
    (0.0, right)
}

#[cfg(test)]
fn host_page_overflow_title_width(text: &str, font_size: f32, clip_guard: f32) -> f32 {
    measure_runtime_text_width(text, font_size) + clip_guard
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_row_frame(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    row: usize,
) -> FrameRect {
    host_page_overflow_row_frame_for_scroll(
        popup,
        row,
        host_page_overflow_clamped_scroll_offset(presentation, popup),
    )
}

fn host_page_overflow_row_frame_for_scroll(
    popup: &FrameRect,
    row: usize,
    scroll_offset: f32,
) -> FrameRect {
    FrameRect {
        x: popup.x + MENU_POPUP_EDGE_INSET,
        y: popup.y + MENU_POPUP_EDGE_INSET + row as f32 * menu_popup_row_stride() - scroll_offset,
        width: (popup.width - MENU_POPUP_EDGE_INSET * 2.0).max(0.0),
        height: MENU_POPUP_ROW_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_visible_row_range(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
) -> std::ops::Range<usize> {
    host_page_overflow_visible_row_range_for_scroll(
        presentation,
        popup,
        host_page_overflow_clamped_scroll_offset(presentation, popup),
    )
}

fn host_page_overflow_visible_row_range_for_scroll(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    scroll_offset: f32,
) -> std::ops::Range<usize> {
    let item_count = presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices
        .len();
    let viewport_height = host_page_overflow_content_viewport_height(popup);
    let stride = menu_popup_row_stride();
    if item_count == 0
        || !viewport_height.is_finite()
        || viewport_height <= 0.0
        || !stride.is_finite()
        || stride <= 0.0
    {
        return 0..0;
    }

    if !scroll_offset.is_finite() {
        return 0..0;
    }
    let first_intersection = (scroll_offset - MENU_POPUP_ROW_HEIGHT) / stride;
    let start = if first_intersection >= 0.0 {
        (first_intersection.floor() as usize).saturating_add(1)
    } else {
        0
    }
    .min(item_count);
    let last_exclusive = ((scroll_offset + viewport_height) / stride).ceil();
    let end = if last_exclusive > 0.0 {
        last_exclusive as usize
    } else {
        0
    }
    .clamp(start, item_count);

    start..end
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_scroll_offset_for_page(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    page_index: usize,
) -> f32 {
    let Some(row) = presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices
        .iter()
        .position(|index| *index == page_index)
    else {
        return host_page_overflow_clamped_scroll_offset(presentation, popup);
    };
    let viewport_height = host_page_overflow_content_viewport_height(popup);
    let current = host_page_overflow_clamped_scroll_offset(presentation, popup);
    let row_top = row as f32 * menu_popup_row_stride();
    let row_bottom = row_top + MENU_POPUP_ROW_HEIGHT;
    let offset = if row_top < current {
        row_top
    } else if row_bottom > current + viewport_height {
        row_bottom - viewport_height
    } else {
        current
    };
    offset.clamp(0.0, host_page_overflow_max_scroll(presentation, popup))
}

/// Applies the shared UI scroll direction to the bounded overflow viewport.
///
/// The caller keeps the popup as the damage region, while this owner keeps the
/// state value within the same content extent used by paint, hit testing, and
/// keyboard reveal.
pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_scroll_offset_for_delta(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    delta: f32,
) -> f32 {
    let current = host_page_overflow_clamped_scroll_offset(presentation, popup);
    if !delta.is_finite() {
        return current;
    }
    (current + delta).clamp(0.0, host_page_overflow_max_scroll(presentation, popup))
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_row_hit_in_popup(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    x: f32,
    y: f32,
) -> Option<HostPageOverflowRowHit> {
    host_page_overflow_row_hit_in_popup_for_scroll(
        presentation,
        popup,
        x,
        y,
        host_page_overflow_clamped_scroll_offset(presentation, popup),
    )
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_row_hit_in_popup_for_scroll(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    x: f32,
    y: f32,
    scroll_offset: f32,
) -> Option<HostPageOverflowRowHit> {
    let scroll_offset = host_page_overflow_clamp_scroll_offset(presentation, popup, scroll_offset);
    let mut viewport = host_page_overflow_content_viewport_frame(popup);
    let scrollbar_gutter =
        host_page_overflow_scrollbar_reserve(presentation, popup.height).min(viewport.width);
    viewport.width = (viewport.width - scrollbar_gutter).max(0.0);
    if !contains(&viewport, x, y) {
        return None;
    }
    for row in host_page_overflow_visible_row_range_for_scroll(presentation, popup, scroll_offset) {
        let frame = host_page_overflow_row_frame_for_scroll(popup, row, scroll_offset);
        if contains(&frame, x, y) {
            return Some(HostPageOverflowRowHit {
                page_index: presentation
                    .host_scene_data
                    .page_chrome
                    .overflow_hidden_tab_indices[row],
                frame,
            });
        }
    }
    None
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_popup_frame_contains(
    popup: &FrameRect,
    x: f32,
    y: f32,
) -> bool {
    contains(popup, x, y)
}

fn contains(frame: &FrameRect, x: f32, y: f32) -> bool {
    x >= frame.x && y >= frame.y && x <= frame.x + frame.width && y <= frame.y + frame.height
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_content_extent(
    presentation: &HostWindowPresentationData,
) -> f32 {
    let item_count = presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices
        .len();
    menu_popup_outer_padding()
        + item_count as f32 * MENU_POPUP_ROW_HEIGHT
        + item_count.saturating_sub(1) as f32 * (menu_popup_row_stride() - MENU_POPUP_ROW_HEIGHT)
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_scroll_content_extent(
    presentation: &HostWindowPresentationData,
) -> f32 {
    (host_page_overflow_content_extent(presentation) - menu_popup_outer_padding()).max(0.0)
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_scrollbar_reserve(
    presentation: &HostWindowPresentationData,
    popup_height: f32,
) -> f32 {
    if host_page_overflow_content_extent(presentation) <= popup_height {
        return 0.0;
    }
    let metrics = current_host_metrics();
    metrics.scrollbar_thickness + metrics.gap_s
}

fn host_page_overflow_vertical_placement(
    anchor: &FrameRect,
    shell_bottom: f32,
    content_height: f32,
) -> HostPageOverflowVerticalPlacement {
    let below_y = anchor.y + anchor.height + MENU_POPUP_ANCHOR_GAP;
    if !below_y.is_finite() || !content_height.is_finite() || content_height <= 0.0 {
        return HostPageOverflowVerticalPlacement {
            y: 0.0,
            height: 0.0,
        };
    }
    if shell_bottom <= 0.0 {
        return HostPageOverflowVerticalPlacement {
            y: below_y,
            height: content_height,
        };
    }
    let shell_top = MENU_POPUP_SHELL_MARGIN.min((shell_bottom * 0.5).max(0.0));
    let shell_bottom = (shell_bottom - MENU_POPUP_SHELL_MARGIN).max(shell_top);
    let below_y = below_y.clamp(shell_top, shell_bottom);
    let above_bottom = (anchor.y - MENU_POPUP_ANCHOR_GAP).clamp(shell_top, shell_bottom);
    let below_height = (shell_bottom - below_y).max(0.0);
    let above_height = (above_bottom - shell_top).max(0.0);
    let place_below = content_height <= below_height
        || (content_height > above_height && below_height >= above_height);
    let available_height = if place_below {
        below_height
    } else {
        above_height
    };
    let height = content_height.min(available_height.max(0.0));
    let y = if place_below {
        below_y
    } else {
        (above_bottom - height).max(shell_top)
    };

    HostPageOverflowVerticalPlacement { y, height }
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_content_viewport_frame(
    popup: &FrameRect,
) -> FrameRect {
    let horizontal_inset = MENU_POPUP_EDGE_INSET.min((popup.width.max(0.0)) * 0.5);
    let vertical_inset = MENU_POPUP_EDGE_INSET.min((popup.height.max(0.0)) * 0.5);
    FrameRect {
        x: popup.x + horizontal_inset,
        y: popup.y + vertical_inset,
        width: (popup.width - horizontal_inset * 2.0).max(0.0),
        height: (popup.height - vertical_inset * 2.0).max(0.0),
    }
}

fn host_page_overflow_content_viewport_height(popup: &FrameRect) -> f32 {
    host_page_overflow_content_viewport_frame(popup).height
}

fn host_page_overflow_clamped_scroll_offset(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
) -> f32 {
    host_page_overflow_clamp_scroll_offset(
        presentation,
        popup,
        presentation.host_page_overflow_menu_state.scroll_offset,
    )
}

fn host_page_overflow_clamp_scroll_offset(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    scroll_offset: f32,
) -> f32 {
    if !scroll_offset.is_finite() {
        return 0.0;
    }
    scroll_offset.clamp(0.0, host_page_overflow_max_scroll(presentation, popup))
}

fn host_page_overflow_max_scroll(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
) -> f32 {
    (host_page_overflow_scroll_content_extent(presentation)
        - host_page_overflow_content_viewport_height(popup))
    .max(0.0)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::ui::retained_host::host_contract::data::{HostPageOverflowMenuStateData, TabData};
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};
    use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

    #[test]
    fn overflow_popup_width_tracks_runtime_title_measure_inside_shell() {
        let shell_width = 640.0;
        let title = "A long hidden editor tab title that should keep its useful glyph space";
        let presentation = overflow_presentation(shell_width, title);

        let popup = host_page_overflow_popup_frame(&presentation)
            .expect("open overflow should provide a popup frame");
        let metrics = current_host_metrics();
        let expected_width =
            (host_page_overflow_title_width(title, metrics.font_body, metrics.text_clip_guard)
                + MENU_POPUP_EDGE_INSET * 2.0
                + MENU_POPUP_TEXT_INSET_X * 2.0
                + metrics.selection_indicator_width
                + metrics.gap_s)
                .max(MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH);

        assert!(popup.width > MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH);
        assert_eq!(popup.width, expected_width);
        assert!(popup.x >= MENU_POPUP_SHELL_MARGIN);
        assert!(popup.x + popup.width <= shell_width - MENU_POPUP_SHELL_MARGIN);
    }

    #[test]
    fn overflow_popup_width_constrains_itself_before_crossing_a_narrow_shell() {
        let shell_width = 96.0;
        let presentation = overflow_presentation(
            shell_width,
            "A long hidden editor tab title that cannot fit in this shell",
        );

        let popup = host_page_overflow_popup_frame(&presentation)
            .expect("open overflow should provide a popup frame");

        assert_eq!(popup.width, shell_width - menu_popup_shell_padding());
        assert_eq!(popup.x, MENU_POPUP_SHELL_MARGIN);
        assert_eq!(popup.x + popup.width, shell_width - MENU_POPUP_SHELL_MARGIN);
    }

    #[test]
    fn overflow_popup_constrains_itself_to_an_offset_shell_frame() {
        let shell_x = 20.0;
        let shell_width = 640.0;
        let mut presentation = overflow_presentation(shell_width, "Hidden tab");
        presentation.host_layout.status_bar_frame.x = shell_x;
        presentation.host_layout.center_band_frame.x = shell_x;
        presentation.host_layout.center_band_frame.width = shell_width;
        presentation.host_scene_data.page_chrome.overflow_frame.x += shell_x;

        let popup = host_page_overflow_popup_frame(&presentation)
            .expect("offset shell should provide a relative popup frame");

        assert!(popup.x >= shell_x + MENU_POPUP_SHELL_MARGIN);
        assert!(
            popup.x + popup.width <= shell_x + shell_width - MENU_POPUP_SHELL_MARGIN,
            "popup right edge should use shell x + width instead of treating width as an absolute x"
        );
    }

    #[test]
    fn overflow_popup_width_does_not_invent_a_one_pixel_shell() {
        let presentation = overflow_presentation(96.0, "Hidden");

        assert_eq!(
            host_page_overflow_popup_width(&presentation, menu_popup_shell_padding(), 0.0),
            0.0
        );
    }

    #[test]
    fn overflow_long_list_natural_width_reserves_its_scrollbar_and_gap() {
        let shell_width = 640.0;
        let mut presentation = overflow_presentation(shell_width, "A naturally measured tab title");
        let unscrolled_width = host_page_overflow_popup_frame(&presentation)
            .expect("short list should provide a popup")
            .width;
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = vec![0; 32];
        let scrolled_width = host_page_overflow_popup_frame(&presentation)
            .expect("long list should provide a bounded popup")
            .width;
        let metrics = current_host_metrics();

        assert_eq!(
            scrolled_width,
            unscrolled_width + metrics.scrollbar_thickness + metrics.gap_s
        );
    }

    #[test]
    fn overflow_natural_width_consumes_the_projected_widest_title_cache() {
        let shell_width = 640.0;
        let mut presentation = overflow_presentation(shell_width, "Short");
        presentation
            .host_scene_data
            .page_chrome
            .overflow_widest_title_width_px = 300.0;
        let metrics = current_host_metrics();
        let expected = 300.0
            + MENU_POPUP_EDGE_INSET * 2.0
            + MENU_POPUP_TEXT_INSET_X * 2.0
            + metrics.selection_indicator_width
            + metrics.gap_s;

        assert_eq!(
            host_page_overflow_popup_frame(&presentation)
                .expect("projected natural width should fit the shell")
                .width,
            expected
        );
    }

    #[test]
    fn overflow_popup_rejects_a_non_finite_anchor_before_layout() {
        let mut presentation = overflow_presentation(240.0, "Hidden");
        presentation.host_scene_data.page_chrome.overflow_frame.x = f32::NAN;

        assert!(host_page_overflow_popup_frame(&presentation).is_none());
    }

    #[test]
    fn overflow_title_width_uses_runtime_text_at_the_supplied_body_size() {
        let text = "WWWW";
        let body_size = EditorTypographyTokens::WORKBENCH_BODY_SIZE;

        assert_eq!(
            host_page_overflow_title_width(text, body_size, 6.0),
            measure_runtime_text_width(text, body_size) + 6.0
        );
    }

    #[test]
    fn overflow_popup_uses_the_space_below_its_anchor_when_it_fits() {
        let placement = host_page_overflow_vertical_placement(
            &FrameRect {
                x: 0.0,
                y: 24.0,
                width: 34.0,
                height: 28.0,
            },
            160.0,
            80.0,
        );

        assert_eq!(placement.y, 55.0);
        assert_eq!(placement.height, 80.0);
    }

    #[test]
    fn overflow_popup_flips_above_when_the_below_side_cannot_hold_its_content() {
        let placement = host_page_overflow_vertical_placement(
            &FrameRect {
                x: 0.0,
                y: 200.0,
                width: 34.0,
                height: 28.0,
            },
            240.0,
            80.0,
        );

        assert_eq!(placement.y, 117.0);
        assert_eq!(placement.height, 80.0);
    }

    #[test]
    fn overflow_popup_clamps_its_viewport_inside_a_tiny_shell_without_overlapping_its_anchor() {
        let placement = host_page_overflow_vertical_placement(
            &FrameRect {
                x: 0.0,
                y: 24.0,
                width: 34.0,
                height: 28.0,
            },
            88.0,
            640.0,
        );

        assert_eq!(placement.y, 55.0);
        assert_eq!(placement.height, 25.0);
        assert!(placement.y + placement.height <= 80.0);
    }

    #[test]
    fn overflow_popup_preserves_its_content_height_when_no_shell_bottom_is_available() {
        let placement = host_page_overflow_vertical_placement(
            &FrameRect {
                x: 0.0,
                y: 24.0,
                width: 34.0,
                height: 28.0,
            },
            0.0,
            80.0,
        );

        assert_eq!(placement.y, 55.0);
        assert_eq!(placement.height, 80.0);
    }

    #[test]
    fn overflow_row_hit_cannot_select_the_clipped_portion_outside_its_popup() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation.host_layout.status_bar_frame.y = 88.0;

        let popup = host_page_overflow_popup_frame(&presentation)
            .expect("tiny shell should still have a bounded popup viewport");
        let first_row = host_page_overflow_row_frame(&presentation, &popup, 0);
        let clipped_y = popup.y + popup.height + 1.0;

        assert!(clipped_y < first_row.y + first_row.height);
        assert!(host_page_overflow_row_hit_in_popup(
            &presentation,
            &popup,
            first_row.x + 1.0,
            clipped_y,
        )
        .is_none());
    }

    #[test]
    fn overflow_row_hit_does_not_activate_through_the_scrollbar_gutter() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = (0..32).collect();
        let popup = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 180.0,
            height: 60.0,
        };
        let viewport = host_page_overflow_content_viewport_frame(&popup);

        assert!(host_page_overflow_row_hit_in_popup(
            &presentation,
            &popup,
            viewport.x + viewport.width - 1.0,
            viewport.y + 1.0,
        )
        .is_none());
    }

    #[test]
    fn overflow_content_viewport_keeps_its_actual_small_height() {
        let popup = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: menu_popup_outer_padding(),
        };

        assert_eq!(host_page_overflow_content_viewport_height(&popup), 0.0);
    }

    #[test]
    fn overflow_popup_is_absent_when_the_shell_cannot_offer_a_usable_content_viewport() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation.host_layout.status_bar_frame.y = 64.0;

        assert!(host_page_overflow_popup_frame(&presentation).is_none());
    }

    #[test]
    fn overflow_popup_is_absent_when_a_collapsed_anchor_leaves_no_horizontal_viewport() {
        let mut presentation = overflow_presentation(
            menu_popup_shell_padding() + MENU_POPUP_EDGE_INSET * 2.0,
            "Hidden tab",
        );
        presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
            x: 0.0,
            y: 24.0,
            width: 1.0,
            height: 28.0,
        };

        assert!(host_page_overflow_popup_frame(&presentation).is_none());
    }

    #[test]
    fn overflow_visible_rows_are_exactly_the_rows_intersecting_the_scrolled_viewport() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = (0..32).collect();
        presentation.host_page_overflow_menu_state.scroll_offset = 60.0;
        let popup = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 180.0,
            height: 60.0,
        };
        let viewport = host_page_overflow_content_viewport_frame(&popup);
        let visible = host_page_overflow_visible_row_range(&presentation, &popup);

        assert!(!visible.is_empty());
        for row in visible.clone() {
            let frame = host_page_overflow_row_frame(&presentation, &popup, row);
            assert!(frame.y < viewport.y + viewport.height);
            assert!(frame.y + frame.height > viewport.y);
        }
        if visible.start > 0 {
            let before = host_page_overflow_row_frame(&presentation, &popup, visible.start - 1);
            assert!(before.y + before.height <= viewport.y);
        }
        if visible.end
            < presentation
                .host_scene_data
                .page_chrome
                .overflow_hidden_tab_indices
                .len()
        {
            let after = host_page_overflow_row_frame(&presentation, &popup, visible.end);
            assert!(after.y >= viewport.y + viewport.height);
        }
    }

    #[test]
    fn overflow_visible_rows_exclude_a_row_that_only_touches_the_viewport_top() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = (0..32).collect();
        presentation.host_page_overflow_menu_state.scroll_offset = MENU_POPUP_ROW_HEIGHT;
        let popup = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 180.0,
            height: 60.0,
        };

        assert_eq!(
            host_page_overflow_visible_row_range(&presentation, &popup).start,
            1
        );
    }

    #[test]
    fn overflow_scroll_clamps_against_row_content_and_the_exact_inner_viewport() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = (0..32).collect();
        let popup = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 180.0,
            height: 60.0,
        };
        let expected = host_page_overflow_scroll_content_extent(&presentation)
            - host_page_overflow_content_viewport_frame(&popup).height;

        assert_eq!(
            host_page_overflow_scroll_offset_for_delta(&presentation, &popup, f32::MAX),
            expected
        );
    }

    #[test]
    fn overflow_scroll_ignores_a_non_finite_input_delta() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = (0..32).collect();
        presentation.host_page_overflow_menu_state.scroll_offset = 30.0;
        let popup = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 180.0,
            height: 60.0,
        };

        assert_eq!(
            host_page_overflow_scroll_offset_for_delta(&presentation, &popup, f32::NAN),
            30.0
        );
    }

    #[test]
    fn overflow_scroll_recovers_a_non_finite_stored_offset() {
        let mut presentation = overflow_presentation(240.0, "Hidden tab");
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = (0..32).collect();
        presentation.host_page_overflow_menu_state.scroll_offset = f32::NAN;
        let popup = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 180.0,
            height: 60.0,
        };

        assert_eq!(
            host_page_overflow_scroll_offset_for_delta(&presentation, &popup, 0.0),
            0.0
        );
    }

    fn overflow_presentation(shell_width: f32, title: &str) -> HostWindowPresentationData {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_layout.status_bar_frame = FrameRect {
            x: 0.0,
            y: 160.0,
            width: shell_width,
            height: 20.0,
        };
        presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
            x: (shell_width - 42.0).max(0.0),
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
            host_page_overflow_title_width(title, metrics.font_body, metrics.text_clip_guard);
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
