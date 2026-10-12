//! 属性标签、值外观和值文字保持局部层序，基础层级来自 fallback 已分配的正文层。
//! 调用者须留足偏移空间，避免把边界层级直接用于局部叠层。

const VALUE_GROUP_OFFSET: i32 = 1;
const FIELD_TEXT_OFFSET: i32 = 1;

pub(super) fn value_group_order(label_order: i32) -> i32 {
    label_order + VALUE_GROUP_OFFSET
}

pub(super) fn field_text_order(field_surface_order: i32) -> i32 {
    field_surface_order + FIELD_TEXT_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
