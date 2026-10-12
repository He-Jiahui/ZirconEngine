//! 标记/轨道、勾号/点、标签和开关滑块的局部层级协议；基础排序边界见 CR-EDITOR-PAINT-0002。

const MARK_CONTENT_ORDER_OFFSET: i32 = 1;
const LABEL_ORDER_OFFSET: i32 = 2;
const TOGGLE_LABEL_ORDER_OFFSET: i32 = 1;
const TOGGLE_THUMB_ORDER_OFFSET: i32 = 2;

pub(super) fn mark_content_order(mark_order: i32) -> i32 {
    mark_order + MARK_CONTENT_ORDER_OFFSET
}

pub(super) fn mark_label_order(mark_order: i32) -> i32 {
    mark_order + LABEL_ORDER_OFFSET
}

pub(super) fn toggle_label_order(track_order: i32) -> i32 {
    track_order + TOGGLE_LABEL_ORDER_OFFSET
}

pub(super) fn toggle_thumb_order(track_order: i32) -> i32 {
    track_order + TOGGLE_THUMB_ORDER_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
