//! 按钮内部叠放协议：表面、状态覆盖、内容、文字；同层命令保留追加次序。基础排序边界问题见 CR-EDITOR-PAINT-0002。

const SURFACE_OVERLAY_OFFSET: i32 = 1;
const CONTENT_OFFSET: i32 = 2;
const LABEL_OFFSET: i32 = 1;

pub(super) fn surface_overlay_order(surface_order: i32) -> i32 {
    surface_order + SURFACE_OVERLAY_OFFSET
}

pub(super) fn content_order(surface_order: i32) -> i32 {
    surface_order + CONTENT_OFFSET
}

pub(super) fn label_order(content_order: i32) -> i32 {
    content_order + LABEL_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
