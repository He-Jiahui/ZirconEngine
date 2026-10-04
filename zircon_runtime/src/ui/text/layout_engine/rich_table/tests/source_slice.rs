use std::sync::Arc;

use crate::text::RichTextFormat;
use crate::ui::text::rich_text::{parse_source_text as try_parse_source_text, UiParsedText};

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
    assert!(slice.runs[0]
        .link()
        .is_some_and(|link| link.target.matches_display("res://docs/second.md")));
    assert!(slice.tables().next().is_none());
}
