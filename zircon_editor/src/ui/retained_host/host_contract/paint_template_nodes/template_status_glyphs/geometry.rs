//! 把信号点与资源图标限定在可用帧内；更外层仍需检查父裁剪与节点边界。

use super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_status_glyph_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_rect(
    rect: &FrameRect,
    size: f32,
) -> FrameRect {
    let width = size.min(rect.width.max(0.0)).max(0.0);
    let height = size.min(rect.height.max(0.0)).max(0.0);
    FrameRect {
        x: rect.x + (rect.width - width).max(0.0) * 0.5,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width,
        height,
    }
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
