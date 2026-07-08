const SEARCH_FIELD_OFFSET: i32 = 1;
const EMPTY_MESSAGE_OFFSET: i32 = 3;
const FIRST_ROW_OFFSET: i32 = 4;
const ROW_STRIDE: i32 = 3;

const SEARCH_ICON_OFFSET: i32 = 1;
const SEARCH_TEXT_OFFSET: i32 = 2;

const ROW_MATCH_INDICATOR_OFFSET: i32 = 1;
const ROW_LABEL_OFFSET: i32 = 2;

pub(super) fn search_field_order(base_order: i32) -> i32 {
    base_order + SEARCH_FIELD_OFFSET
}

pub(super) fn empty_message_order(base_order: i32) -> i32 {
    base_order + EMPTY_MESSAGE_OFFSET
}

pub(super) fn search_icon_order(search_order: i32) -> i32 {
    search_order + SEARCH_ICON_OFFSET
}

pub(super) fn search_text_order(search_order: i32) -> i32 {
    search_order + SEARCH_TEXT_OFFSET
}

pub(super) fn row_order(base_order: i32, row: usize) -> i32 {
    base_order + FIRST_ROW_OFFSET + row as i32 * ROW_STRIDE
}

pub(super) fn row_match_indicator_order(row_order: i32) -> i32 {
    row_order + ROW_MATCH_INDICATOR_OFFSET
}

pub(super) fn row_label_order(row_order: i32) -> i32 {
    row_order + ROW_LABEL_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_row_order_advances_by_row_stride() {
        assert_eq!(row_order(10, 0), 14);
        assert_eq!(row_order(10, 1), 17);
        assert_eq!(row_order(10, 2), 20);
    }
}
