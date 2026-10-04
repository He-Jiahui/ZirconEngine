use crate::core::framework::text::TextLayoutError;
use crate::text::{SharedTextLayoutSession, TextLayoutGeometryBudget};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiTextDirection, UiTextRange,
};

use super::line_box::MIN_TEXT_FONT_SIZE;

/// Owns the sole safe-publication fallback for shaping and layout failures.
///
/// The result deliberately has no lines or artifact. Callers must keep it out of frame and
/// persistent caches, so a deferred generation cannot reuse invalid geometry after recovery.
pub(in crate::ui::text) fn text_layout_error_layout(
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    font_size: f32,
    line_height: f32,
    source_len: usize,
    error: &TextLayoutError,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    provider.record_layout_error(error);
    let geometry_budget = provider.geometry_budget();
    let font_size = admitted_failure_extent(
        font_size,
        MIN_TEXT_FONT_SIZE.min(geometry_budget.max_axis_extent()),
        geometry_budget,
    );
    let line_height = admitted_failure_extent(line_height, font_size, geometry_budget);
    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height,
        measured_width: 0.0,
        measured_height: line_height,
        source_range: UiTextRange {
            start: 0,
            end: source_len,
        },
        lines: Vec::new(),
        boxes: Vec::new(),
        overflow_clipped: true,
        editable: None,
        rich_text_artifact: None,
    }
}

fn admitted_failure_extent(
    value: f32,
    fallback: f32,
    geometry_budget: TextLayoutGeometryBudget,
) -> f32 {
    if value >= fallback && geometry_budget.admit_axis_extent(value).is_ok() {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "tests/failure_layout.rs"]
mod tests;
