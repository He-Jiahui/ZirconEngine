use crate::core::framework::text::TextLayoutError;
use crate::text::shaping::TextLayoutOutcome;
use crate::text::shaping::TextShapingOutcome;
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout},
};

use super::super::super::rich_text::UiParsedText;
use super::super::layout_parsed_text_with_provider_outcome;

pub(super) fn layout_range_with_provider(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip: UiFrame,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiResolvedTextLayout> {
    let local = match slice_parsed(parsed, range.clone()) {
        Ok(local) => local,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    layout_parsed_text_with_provider_outcome(&local, style, frame, Some(clip), provider).map(
        |mut layout| {
            shift_layout_source_ranges(&mut layout, range.start);
            layout
        },
    )
}

pub(super) fn slice_parsed(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
) -> Result<UiParsedText, TextLayoutError> {
    slice_parsed_with_table_depth(parsed, range, None)
}

pub(super) fn slice_parsed_with_table_depth(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    parent_table_depth: Option<u16>,
) -> Result<UiParsedText, TextLayoutError> {
    parsed.project_range(range, parent_table_depth)
}

pub(super) fn shift_layout_source_ranges(layout: &mut UiResolvedTextLayout, offset: usize) {
    layout.source_range.start += offset;
    layout.source_range.end += offset;
    for text_box in &mut layout.boxes {
        text_box.range.start += offset;
        text_box.range.end += offset;
    }
    for line in &mut layout.lines {
        line.source_range.start += offset;
        line.source_range.end += offset;
        for run in &mut line.runs {
            run.source_range.start += offset;
            run.source_range.end += offset;
        }
    }
}

#[cfg(test)]
#[path = "tests/source_slice.rs"]
mod tests;
