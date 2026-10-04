//! 下拉框内部层级：标签与箭头在表面之上；基础排序边界见 CR-EDITOR-PAINT-0002。

const LABEL_OFFSET: i32 = 2;
const CHEVRON_OFFSET: i32 = 3;

pub(super) fn label_order(surface_order: i32) -> i32 {
    surface_order + LABEL_OFFSET
}

pub(super) fn chevron_order(surface_order: i32) -> i32 {
    surface_order + CHEVRON_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
