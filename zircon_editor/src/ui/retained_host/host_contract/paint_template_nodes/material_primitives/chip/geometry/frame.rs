use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::chip_is_small;
use super::metrics::{chip_bounded_extent, CHIP_MEDIUM_HEIGHT, CHIP_SMALL_HEIGHT};

/// 将 Chip 的标准高度收进父节点边界，供所有子层共用同一可绘帧。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let available_height = chip_bounded_extent(rect.height);
    let target_height = chip_height(node).min(available_height);
    FrameRect {
        x: rect.x,
        y: rect.y + (available_height - target_height) * 0.5,
        width: chip_bounded_extent(rect.width),
        height: target_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let half_extent = chip_bounded_extent(rect.width).min(chip_bounded_extent(rect.height)) * 0.5;
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    if configured.is_finite() && configured > 0.0 {
        configured.min(half_extent)
    } else {
        half_extent
    }
}

fn chip_height(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        CHIP_SMALL_HEIGHT
    } else {
        CHIP_MEDIUM_HEIGHT
    }
}

#[cfg(test)]
#[path = "tests/frame.rs"]
mod tests;
