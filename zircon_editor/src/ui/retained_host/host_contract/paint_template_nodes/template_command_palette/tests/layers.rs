use super::*;

#[test]
fn command_palette_row_order_advances_by_row_stride() {
    assert_eq!(row_order(10, 0), 14);
    assert_eq!(row_order(10, 1), 17);
    assert_eq!(row_order(10, 2), 20);
}
