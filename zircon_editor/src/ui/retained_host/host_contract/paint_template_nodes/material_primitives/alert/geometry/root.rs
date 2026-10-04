use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::alert_bounded_extent;

// 根框由命令序列消费；只收紧宽高，不扩张宿主分配的节点范围。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: alert_bounded_extent(rect.width),
        height: alert_bounded_extent(rect.height),
    }
}

#[cfg(test)]
#[path = "tests/root.rs"]
mod tests;
