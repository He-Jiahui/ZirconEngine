//! 轨道、填充、刻度、标签、双滑块与数值浮层的内部叠放协议；基础排序边界见 CR-EDITOR-PAINT-0002。

const TRACK_FILL_ORDER_OFFSET: i32 = 1;
const TICK_ORDER_OFFSET: i32 = 2;
const LABEL_ORDER_OFFSET: i32 = 3;
const RANGE_MIN_THUMB_ORDER_OFFSET: i32 = 3;
const PRIMARY_THUMB_ORDER_OFFSET: i32 = 4;
const VALUE_SURFACE_ORDER_OFFSET: i32 = 5;
const INNER_TEXT_ORDER_OFFSET: i32 = 1;
const THUMB_BODY_ORDER_OFFSET: i32 = 1;

pub(super) fn track_fill_order(track_order: i32) -> i32 {
    track_order + TRACK_FILL_ORDER_OFFSET
}

pub(super) fn tick_order(track_order: i32) -> i32 {
    track_order + TICK_ORDER_OFFSET
}

pub(super) fn label_order(track_order: i32) -> i32 {
    track_order + LABEL_ORDER_OFFSET
}

pub(super) fn range_min_thumb_order(track_order: i32) -> i32 {
    track_order + RANGE_MIN_THUMB_ORDER_OFFSET
}

pub(super) fn primary_thumb_order(track_order: i32) -> i32 {
    track_order + PRIMARY_THUMB_ORDER_OFFSET
}

pub(super) fn value_surface_order(track_order: i32) -> i32 {
    track_order + VALUE_SURFACE_ORDER_OFFSET
}

pub(super) fn inner_text_order(surface_order: i32) -> i32 {
    surface_order + INNER_TEXT_ORDER_OFFSET
}

pub(super) fn thumb_body_order(halo_order: i32) -> i32 {
    halo_order + THUMB_BODY_ORDER_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
