//! 通知面板的物理位置保留DPI小数；尺寸有效性由commands入口继续判定。

use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width.max(1.0),
        height: rect.height.max(1.0),
    }
}

#[cfg(test)]
#[path = "tests/common.rs"]
mod tests;
