//! 表面、分隔线、单元格、操作槽和图标按局部层级排序；表格入口须提供可容纳偏移的基础层。

const SEPARATOR_OFFSET: i32 = 1;
const CELLS_OFFSET: i32 = 2;
const ACTION_SLOT_OFFSET: i32 = 3;
const ACTION_ICON_OFFSET: i32 = 1;

pub(super) fn separator_order(surface_order: i32) -> i32 {
    surface_order + SEPARATOR_OFFSET
}

pub(super) fn cells_order(surface_order: i32) -> i32 {
    surface_order + CELLS_OFFSET
}

pub(super) fn action_slot_order(surface_order: i32) -> i32 {
    surface_order + ACTION_SLOT_OFFSET
}

pub(super) fn action_icon_order(slot_order: i32) -> i32 {
    slot_order + ACTION_ICON_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
