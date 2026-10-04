use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout, UiTextCaret, UiTextRange},
};

/// Returns a caret frame for an already-resolved text layout using shaped source-range metrics
/// whenever the layout line is simple enough to make source and visual ranges equivalent.
pub fn text_caret_frame_for_layout(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    source_text: &str,
    style: &UiResolvedStyle,
) -> Option<UiFrame> {
    crate::ui::text::caret_frame_for_text_layout_with_source_metrics(
        layout,
        caret,
        source_text,
        style,
    )
}

/// Returns source-range frames for an already-resolved text layout using shaped source-range
/// metrics when available, with the text geometry owner retaining fallback behavior.
pub fn text_range_frames_for_layout(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    source_text: &str,
    style: &UiResolvedStyle,
) -> Vec<UiFrame> {
    crate::ui::text::text_range_frames_for_text_layout_with_source_metrics(
        layout,
        range,
        source_text,
        style,
    )
}

#[cfg(test)]
#[path = "tests/text_geometry.rs"]
mod tests;
