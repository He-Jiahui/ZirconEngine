//! 树行内部的面、缩进、展开符、对象、标题和操作槽共享基础层级，但必须保持局部先后。
//! 调用方须留出所有偏移的 i32 余量；该模块不会验证上游层级边界。

const INDENT_GUIDES_OFFSET: i32 = 1;
const DISCLOSURE_OFFSET: i32 = 2;
const OBJECT_ICON_OFFSET: i32 = 3;
const LABEL_OFFSET: i32 = 4;
const ACTION_SLOT_OFFSET: i32 = 5;
const PRIMARY_ACTION_ICON_OFFSET: i32 = 1;
const SECONDARY_ACTION_SLOT_OFFSET: i32 = 2;
const SECONDARY_ACTION_ICON_OFFSET: i32 = 3;

pub(super) fn indent_guides_order(surface_order: i32) -> i32 {
    surface_order + INDENT_GUIDES_OFFSET
}

pub(super) fn disclosure_order(surface_order: i32) -> i32 {
    surface_order + DISCLOSURE_OFFSET
}

pub(super) fn object_icon_order(surface_order: i32) -> i32 {
    surface_order + OBJECT_ICON_OFFSET
}

pub(super) fn label_order(surface_order: i32) -> i32 {
    surface_order + LABEL_OFFSET
}

pub(super) fn action_slot_order(surface_order: i32) -> i32 {
    surface_order + ACTION_SLOT_OFFSET
}

pub(super) fn primary_action_icon_order(action_order: i32) -> i32 {
    action_order + PRIMARY_ACTION_ICON_OFFSET
}

pub(super) fn secondary_action_slot_order(action_order: i32) -> i32 {
    action_order + SECONDARY_ACTION_SLOT_OFFSET
}

pub(super) fn secondary_action_icon_order(action_order: i32) -> i32 {
    action_order + SECONDARY_ACTION_ICON_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
