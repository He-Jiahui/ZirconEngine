use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::chip_is_small;
use super::metrics::{CHIP_MEDIUM_HEIGHT, CHIP_SMALL_HEIGHT};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let target_height = chip_height(node).min(rect.height.max(1.0)).round();
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - target_height).max(0.0) * 0.5).round(),
        width: rect.width.round().max(1.0),
        height: target_height.max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    if configured > 0.0 {
        configured.min(rect.height * 0.5)
    } else {
        rect.height * 0.5
    }
}

fn chip_height(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        CHIP_SMALL_HEIGHT
    } else {
        CHIP_MEDIUM_HEIGHT
    }
}
