//! chip 的局部叠放约定：标签与箭头在表面上方；基础层级边界见 CR-EDITOR-PAINT-0002。

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
