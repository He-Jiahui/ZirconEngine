use crate::core::framework::text::TextLayoutError;
use crate::text::SharedTextLayoutSession;
use crate::text::shaping::TextLayoutOutcome;
use crate::text::shaping::TextShapingOutcome;
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
mod tests {
    use std::sync::Arc;

    use crate::text::RichTextFormat;
    use crate::ui::text::rich_text::{UiParsedText, parse_source_text as try_parse_source_text};

    use super::slice_parsed_with_table_depth;

    fn parse_source_text(text: &str, format: RichTextFormat) -> UiParsedText {
        try_parse_source_text(text, format).expect("test text fits parser budgets")
    }

    #[test]
    fn table_cell_projection_reuses_parent_compiled_artifact_and_metadata() {
        let parsed = parse_source_text(
            "[table=2][cell][b]first[/b][/cell][cell][url=res://docs/second.md]second[/url][/cell][/table]",
            RichTextFormat::BbCodeV1,
        );
        let table = parsed.rich.parsed().tables.first().expect("parsed table");
        let cell = table.cells.get(1).expect("second parsed cell");
        let slice = slice_parsed_with_table_depth(
            &parsed,
            cell.byte_range.0 as usize..cell.byte_range.1 as usize,
            Some(table.depth),
        )
        .expect("valid table cell projection");

        assert!(Arc::ptr_eq(&parsed.rich, &slice.rich));
        assert_eq!(slice.text(), "second");
        assert_eq!(slice.runs.len(), 1);
        assert!(
            slice.runs[0]
                .link()
                .is_some_and(|link| link.target.matches_display("res://docs/second.md"))
        );
        assert!(slice.tables().next().is_none());
    }
}
