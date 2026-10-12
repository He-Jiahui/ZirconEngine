//! 图标按钮内部层级协议：图标放在表面上方；基础排序边界问题见 CR-EDITOR-PAINT-0002。

const GLYPH_OFFSET: i32 = 2;

pub(super) fn glyph_order(surface_order: i32) -> i32 {
    surface_order + GLYPH_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
