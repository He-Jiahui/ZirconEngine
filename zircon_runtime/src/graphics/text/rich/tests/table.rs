use crate::core::framework::render::{RichTable, RichTextFormat};

use super::parse_rich_text;

#[test]
fn text_rich_bbcode_table_emits_row_major_cell_ranges() {
    let parsed = parse_rich_text(
        "[table=2][cell]name[/cell][cell][b]value[/b][/cell][/table]",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "namevalue");
    assert_eq!(parsed.tables.len(), 1);
    let table = &parsed.tables[0];
    assert_eq!(table.byte_range, (0, 9));
    assert_eq!(table.columns.len(), 2);
    assert_eq!(table.cells.len(), 2);
    assert_eq!(table.cells[0].byte_range, (0, 4));
    assert_eq!(cell_grid(table)[0], (0, 0, 1, 1));
    assert_eq!(table.cells[1].byte_range, (4, 9));
    assert_eq!(cell_grid(table)[1], (0, 1, 1, 1));
    assert_eq!(parsed.runs[1].style.weight, Some(700));
}

#[test]
fn text_rich_bbcode_table_is_a_block_between_surrounding_text() {
    let parsed = parse_rich_text(
        "before[table=2][cell]A[/cell][cell]B[/cell][/table]after",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.text, "before\nAB\nafter");
    assert_eq!(parsed.tables[0].byte_range, (7, 9));
    assert_eq!(parsed.tables[0].cells[0].byte_range, (7, 8));
    assert_eq!(parsed.tables[0].cells[1].byte_range, (8, 9));
}

#[test]
fn text_rich_bbcode_table_clamps_hostile_column_count() {
    let parsed = parse_rich_text(
        "[table=999999][cell]safe[/cell][/table]",
        RichTextFormat::BbCode,
    );

    assert_eq!(parsed.tables[0].columns.len(), 64);
    assert_eq!(parsed.tables[0].cells.len(), 1);
}

#[test]
fn text_rich_bbcode_cell_outside_table_degrades_to_inner_text() {
    let parsed = parse_rich_text("a[cell]safe[/cell]b", RichTextFormat::BbCode);

    assert_eq!(parsed.text, "asafeb");
    assert!(parsed.tables.is_empty());
}

#[test]
fn text_rich_bbcode_table_column_expand_ratio_is_shared_by_column() {
    let parsed = parse_rich_text(
        "[table=2][cell expand=3 shrink=false]wide[/cell][cell]plain[/cell][/table]",
        RichTextFormat::BbCode,
    );

    assert!(parsed.tables[0].columns[0].expand);
    assert_eq!(parsed.tables[0].columns[0].expand_ratio, 3);
    assert!(!parsed.tables[0].columns[0].shrink);
    assert!(!parsed.tables[0].columns[1].expand);
}

#[test]
fn text_rich_bbcode_table_spans_skip_occupied_slots_and_clamp_to_the_row() {
    let parsed = parse_rich_text(
        "[table=3][cell rowspan=2]A[/cell][cell colspan=9]B[/cell][cell]C[/cell][/table]",
        RichTextFormat::BbCode,
    );

    assert_eq!(
        cell_grid(&parsed.tables[0]),
        vec![(0, 0, 1, 2), (0, 1, 2, 1), (1, 1, 1, 1)]
    );
}

#[test]
fn text_rich_bbcode_table_colspan_stops_before_an_occupied_slot() {
    let parsed = parse_rich_text(
        "[table=3][cell]A[/cell][cell rowspan=2]B[/cell][cell]C[/cell][cell colspan=2]D[/cell][cell]E[/cell][/table]",
        RichTextFormat::BbCode,
    );

    assert_eq!(
        cell_grid(&parsed.tables[0]),
        vec![
            (0, 0, 1, 1),
            (0, 1, 1, 2),
            (0, 2, 1, 1),
            (1, 0, 1, 1),
            (1, 2, 1, 1),
        ]
    );
}

#[test]
fn text_rich_bbcode_table_span_values_are_bounded_and_invalid_values_default_to_one() {
    let parsed = parse_rich_text(
        "[table=2][cell colspan=0 rowspan=-3]A[/cell][cell colspan=nope rowspan=999999]B[/cell][/table]",
        RichTextFormat::BbCode,
    );

    assert_eq!(
        cell_grid(&parsed.tables[0]),
        vec![(0, 0, 1, 1), (0, 1, 1, 64)]
    );
}

#[test]
fn text_rich_bbcode_spanning_cell_configures_every_covered_column() {
    let parsed = parse_rich_text(
        "[table=3][cell colspan=2 expand=3 shrink=false]wide[/cell][cell]plain[/cell][/table]",
        RichTextFormat::BbCode,
    );

    let table = &parsed.tables[0];
    for column in &table.columns[..2] {
        assert!(column.expand);
        assert_eq!(column.expand_ratio, 3);
        assert!(!column.shrink);
    }
    assert!(!table.columns[2].expand);
}

#[test]
fn text_rich_bbcode_table_cell_box_options_parse_without_renderer_semantics() {
    let parsed = parse_rich_text(
        "[table=2][cell border=#73D7FF bg=#12202C,#182F3D padding=8,4,12,6]A[/cell][/table]",
        RichTextFormat::BbCode,
    );

    let style = &parsed.tables[0].cells[0].box_style;
    let padding = style.padding.expect("authored cell padding");
    assert_eq!((padding.left, padding.top), (8.0, 4.0));
    assert_eq!((padding.right, padding.bottom), (12.0, 6.0));
    assert!(style.border_color.is_some());
    assert!(style.odd_row_background.is_some());
    assert!(style.even_row_background.is_some());
    assert_ne!(style.odd_row_background, style.even_row_background);
}

#[test]
fn text_rich_bbcode_table_cell_box_options_bound_hostile_values_atomically() {
    let parsed = parse_rich_text(
        "[table=3][cell bg=#123 padding=-3,4,999999,6]bounded[/cell][cell border=invalid bg=#123,#ggg padding=1,2,3]invalid[/cell][cell padding=1,NaN,3,4]nonfinite[/cell][/table]",
        RichTextFormat::BbCode,
    );

    let bounded = &parsed.tables[0].cells[0].box_style;
    let padding = bounded.padding.expect("finite tuple remains authored");
    assert_eq!(padding.left, 0.0);
    assert_eq!(padding.top, 4.0);
    assert_eq!(padding.right, 4096.0);
    assert_eq!(padding.bottom, 6.0);
    assert_eq!(bounded.odd_row_background, bounded.even_row_background);

    let invalid = &parsed.tables[0].cells[1].box_style;
    assert!(invalid.padding.is_none());
    assert!(invalid.border_color.is_none());
    assert!(invalid.odd_row_background.is_none());
    assert!(invalid.even_row_background.is_none());

    assert!(parsed.tables[0].cells[2].box_style.padding.is_none());
}

fn cell_grid(table: &RichTable) -> Vec<(u32, u16, u16, u16)> {
    table
        .cells
        .iter()
        .map(|cell| {
            (
                cell.row_index,
                cell.column_index,
                cell.column_span,
                cell.row_span,
            )
        })
        .collect()
}
