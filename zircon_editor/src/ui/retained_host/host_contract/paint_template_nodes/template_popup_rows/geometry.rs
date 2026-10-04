//! 弹层行的有限尺寸与包含性合同；部分clip仍可绘制，但行内文本与分隔线要完整属于自身行框。

use super::super::super::data::FrameRect;

pub(super) fn has_paintable_popup_row_extent(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    has_paintable_popup_row_extent(outer)
        && has_paintable_popup_row_extent(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
