use super::*;

#[test]
fn table_stack_collapses_without_room_for_its_header() {
    assert_eq!(
        compact_table_stack_height(BROWSER_CONTENT_TABLE_HEADER_HEIGHT - 1.0, 4),
        0.0
    );
    assert_eq!(compact_table_stack_height(f32::NAN, 4), 0.0);
}

#[test]
fn table_stack_never_exceeds_its_available_height() {
    let height = compact_table_stack_height(BROWSER_CONTENT_TABLE_HEADER_HEIGHT + 4.0, 4);
    assert_eq!(height, BROWSER_CONTENT_TABLE_HEADER_HEIGHT + 4.0);
}
