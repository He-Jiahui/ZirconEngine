//! chip 外框与标签槽沿用布局的浮点坐标；内容先按可用宽度收缩，像素级裁剪交给命令的 clip。

use super::super::super::data::FrameRect;
use super::metrics::{chip_line_height, chip_text_left};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_chip_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

/// caller 按可绘箭头决定 right_reserve；调用者仍须检查所得文字槽有正宽且与 clip 相交。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_rect(
    rect: &FrameRect,
    right_reserve: f32,
) -> FrameRect {
    let line_height = chip_line_height();
    let text_left = chip_text_left();
    FrameRect {
        x: rect.x + text_left,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - text_left - right_reserve).max(0.0),
        height: line_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    // Chips are rounded interactive surfaces; keep fractional edges for final analytic coverage.
    rect.clone()
}
