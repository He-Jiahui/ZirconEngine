const POPUP_BACKGROUND_ORDER_OFFSET: i32 = 10_000;
const POPUP_ROW_SURFACE_ORDER_OFFSET: i32 = POPUP_BACKGROUND_ORDER_OFFSET + 1;
const POPUP_SEPARATOR_ORDER_OFFSET: i32 = POPUP_BACKGROUND_ORDER_OFFSET + 2;
const POPUP_TEXT_ORDER_OFFSET: i32 = POPUP_BACKGROUND_ORDER_OFFSET + 3;
const POPUP_ADORNMENT_ORDER_OFFSET: i32 = POPUP_BACKGROUND_ORDER_OFFSET + 4;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_background_order(
    order: i32,
) -> i32 {
    order + POPUP_BACKGROUND_ORDER_OFFSET
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_surface_order(
    row_order: i32,
) -> i32 {
    row_order + POPUP_ROW_SURFACE_ORDER_OFFSET
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_separator_order(
    row_order: i32,
) -> i32 {
    row_order + POPUP_SEPARATOR_ORDER_OFFSET
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_text_order(
    row_order: i32,
) -> i32 {
    row_order + POPUP_TEXT_ORDER_OFFSET
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_base_order(
    order: i32,
    row: usize,
) -> i32 {
    order + row as i32
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_adornment_order(
    row_order: i32,
) -> i32 {
    row_order + POPUP_ADORNMENT_ORDER_OFFSET
}
