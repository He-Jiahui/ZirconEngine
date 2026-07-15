use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextAlign, UiTextDirection,
    UiTextRange, UiTextWritingMode,
};

use super::measure_text_source_range_width;

#[derive(Clone, Copy)]
pub(super) struct SourceMeasureContext<'a> {
    pub(super) text: &'a str,
    pub(super) style: &'a UiResolvedStyle,
}

pub(super) fn measured_source_prefix_width(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    visual_offset: usize,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> Option<f32> {
    let measure_context = measure_context?;
    if !line_accepts_source_measure(layout, line, measure_context.text) {
        return None;
    }

    let range = source_prefix_range_for_visual_offset(line, visual_offset);
    let width = measure_text_source_range_width(measure_context.text, measure_context.style, range);
    width.is_finite().then_some(width.max(0.0))
}

pub(super) fn source_prefix_range_for_visual_offset(
    line: &UiResolvedTextLine,
    visual_offset: usize,
) -> UiTextRange {
    let local_end = grapheme_floor(line.text.as_str(), visual_offset.min(line.text.len()));
    UiTextRange {
        start: line.source_range.start,
        end: line
            .source_range
            .start
            .saturating_add(local_end)
            .min(line.source_range.end),
    }
}

pub(super) fn line_accepts_source_measure(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    source_text: &str,
) -> bool {
    if matches!(layout.text_align, UiTextAlign::Justify)
        || !matches!(layout.writing_mode, UiTextWritingMode::HorizontalTb)
        || !matches!(layout.direction, UiTextDirection::LeftToRight)
        || !matches!(line.direction, UiTextDirection::LeftToRight)
        || line.ellipsized
        || line.text.contains('\t')
    {
        return false;
    }

    let Some(source_slice) = source_text.get(line.source_range.start..line.source_range.end) else {
        return false;
    };
    if source_slice != line.text {
        return false;
    }

    let [run] = line.runs.as_slice() else {
        return false;
    };
    run.source_range == line.source_range
        && run.visual_range == line.visual_range
        && run.text == line.text
        && matches!(run.direction, UiTextDirection::LeftToRight)
}

fn grapheme_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return start;
        }
        if start >= offset {
            break;
        }
    }
    offset
}
