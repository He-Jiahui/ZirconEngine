//! 轴值文字在同一字段表面上一层绘制；调用者提供的基础 order 需要给此偏移保留范围。

const VALUE_TEXT_OFFSET: i32 = 1;

pub(super) fn value_text_order(surface_order: i32) -> i32 {
    surface_order + VALUE_TEXT_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
