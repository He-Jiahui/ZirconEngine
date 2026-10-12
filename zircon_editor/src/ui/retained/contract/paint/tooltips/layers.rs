//! 以阴影为基础安排 tooltip 内部层级；正文紧随标题，箭头和 info 标记在气泡上方。
//! 上游须为相对偏移预留 order 空间，不可把这些值当作独立绝对层级。

const BUBBLE_OFFSET: i32 = 1;
const TEXT_OFFSET: i32 = 2;
const ARROW_OFFSET: i32 = 3;
const ICON_OFFSET: i32 = 4;
const BODY_OFFSET: i32 = 1;

pub(super) fn bubble_order(shadow_order: i32) -> i32 {
    shadow_order + BUBBLE_OFFSET
}

pub(super) fn text_order(shadow_order: i32) -> i32 {
    shadow_order + TEXT_OFFSET
}

pub(super) fn arrow_order(shadow_order: i32) -> i32 {
    shadow_order + ARROW_OFFSET
}

pub(super) fn icon_order(shadow_order: i32) -> i32 {
    shadow_order + ICON_OFFSET
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn body_order(
    title_order: i32,
) -> i32 {
    title_order + BODY_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
