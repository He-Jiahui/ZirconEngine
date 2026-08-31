use std::ops::Range;

pub(in crate::ui::retained_host::host_contract) use crate::ui::retained_host::menu_popup_contract::{
    MENU_POPUP_ROW_GAP, MENU_POPUP_ROW_HEIGHT,
};
use crate::ui::retained_host::menu_popup_contract::{
    MENU_POPUP_ANCHOR_GAP as SHARED_MENU_POPUP_ANCHOR_GAP, MENU_POPUP_EDGE_MARGIN,
    MENU_POPUP_MIN_HEIGHT, MENU_POPUP_PADDING,
};

use super::paint_text::measure_runtime_text_width;
use super::paint_theme::current_host_metrics;

pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_EDGE_INSET: f32 =
    MENU_POPUP_PADDING;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_ANCHOR_GAP: f32 =
    SHARED_MENU_POPUP_ANCHOR_GAP;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_SHELL_MARGIN: f32 =
    MENU_POPUP_EDGE_MARGIN;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_MIN_VISIBLE_HEIGHT: f32 =
    MENU_POPUP_MIN_HEIGHT;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_TEXT_INSET_X: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract) const MENU_POPUP_TEXT_INSET_Y: f32 = 6.0;

pub(crate) fn menu_popup_text_width(text: &str) -> f32 {
    let metrics = current_host_metrics();
    measure_runtime_text_width(text, metrics.font_body) + metrics.text_clip_guard
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_row_stride() -> f32 {
    MENU_POPUP_ROW_HEIGHT + MENU_POPUP_ROW_GAP
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_visible_row_range(
    item_count: usize,
    viewport_height: f32,
    scroll_offset: f32,
    first_row_offset: f32,
) -> Range<usize> {
    let stride = menu_popup_row_stride();
    if item_count == 0
        || !viewport_height.is_finite()
        || viewport_height <= 0.0
        || !scroll_offset.is_finite()
        || !first_row_offset.is_finite()
        || !stride.is_finite()
        || stride <= 0.0
    {
        return 0..0;
    }

    let first_intersection = (scroll_offset - first_row_offset - MENU_POPUP_ROW_HEIGHT) / stride;
    let last_exclusive = ((scroll_offset + viewport_height - first_row_offset) / stride).ceil();
    if !first_intersection.is_finite() || !last_exclusive.is_finite() {
        return 0..0;
    }
    let start = if first_intersection >= 0.0 {
        (first_intersection.floor() as usize).saturating_add(1)
    } else {
        0
    }
    .min(item_count);
    let end = if last_exclusive > 0.0 {
        last_exclusive as usize
    } else {
        0
    }
    .clamp(start, item_count);

    start..end
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_outer_padding() -> f32 {
    MENU_POPUP_EDGE_INSET * 2.0
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_shell_padding() -> f32 {
    MENU_POPUP_SHELL_MARGIN * 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_row_range_keeps_only_strict_viewport_intersections() {
        assert_eq!(menu_popup_visible_row_range(32, 60.0, 60.0, 0.0), 2..4);
        assert_eq!(menu_popup_visible_row_range(32, 60.0, 28.0, 0.0), 1..3);
        assert_eq!(menu_popup_visible_row_range(32, 60.0, 0.0, 6.0), 0..2);
    }

    #[test]
    fn visible_row_range_rejects_invalid_viewports_and_offsets() {
        assert_eq!(menu_popup_visible_row_range(0, 60.0, 0.0, 0.0), 0..0);
        assert_eq!(menu_popup_visible_row_range(8, 0.0, 0.0, 0.0), 0..0);
        assert_eq!(menu_popup_visible_row_range(8, 60.0, f32::NAN, 0.0), 0..0);
    }
}
