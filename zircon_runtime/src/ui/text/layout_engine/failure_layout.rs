use crate::core::framework::text::TextLayoutError;
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiTextDirection, UiTextRange,
};

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
    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height,
        measured_width: 0.0,
        measured_height: line_height.max(0.0),
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
