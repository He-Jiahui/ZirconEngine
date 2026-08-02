use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout},
};

use super::super::super::rich_text::UiParsedText;
use super::super::layout_parsed_text_with_provider;

pub(super) fn layout_range_with_provider(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip: UiFrame,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    let start = range.start.min(parsed.text().len());
    let end = range.end.min(parsed.text().len()).max(start);
    let local = slice_parsed(parsed, start..end);
    let mut layout = layout_parsed_text_with_provider(&local, style, frame, Some(clip), provider);
    shift_layout_source_ranges(&mut layout, start);
    layout
}

pub(super) fn slice_parsed(parsed: &UiParsedText, range: std::ops::Range<usize>) -> UiParsedText {
    slice_parsed_with_table_depth(parsed, range, None)
}

pub(super) fn slice_parsed_with_table_depth(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    parent_table_depth: Option<u16>,
) -> UiParsedText {
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
    use crate::ui::text::parse_source_text;

    use super::slice_parsed_with_table_depth;

    #[test]
    fn table_cell_projection_reuses_parent_compiled_artifact_and_metadata() {
        let parsed = parse_source_text(
            "[table=2][cell][b]first[/b][/cell][cell][url=res://docs/second.md]second[/url][/cell][/table]",
            RichTextFormat::BbCode,
        );
        let table = parsed.rich.parsed().tables.first().expect("parsed table");
        let cell = table.cells.get(1).expect("second parsed cell");
        let slice = slice_parsed_with_table_depth(
            &parsed,
            cell.byte_range.0 as usize..cell.byte_range.1 as usize,
            Some(table.depth),
        );

        assert!(Arc::ptr_eq(&parsed.rich, &slice.rich));
        assert_eq!(slice.text(), "second");
        assert_eq!(slice.runs.len(), 1);
        assert_eq!(
            slice.runs[0].link().map(|link| link.href.as_str()),
            Some("res://docs/second.md")
        );
        assert!(slice.tables().next().is_none());
    }
}
