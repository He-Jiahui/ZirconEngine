//! 字段内部视觉顺序：表面、搜索前缀、清除/步进、文字；层级必须与基础节点排序预算一致。

pub(super) const SEARCH_GLYPH_OFFSET: i32 = 1;
pub(super) const SEARCH_CLEAR_ACTION_OFFSET: i32 = 2;
pub(super) const STEPPER_OFFSET: i32 = 2;
pub(super) const TEXT_OFFSET: i32 = 3;

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
