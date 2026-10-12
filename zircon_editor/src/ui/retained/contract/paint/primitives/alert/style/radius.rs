use super::tokens::ALERT_DEFAULT_RADIUS;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

// 表面命令读取声明圆角或默认半径，并按当前框尺寸收紧，避免边角超出根节点。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    let radius = if configured > 0.0 {
        configured
    } else {
        ALERT_DEFAULT_RADIUS
    };
    radius.min(rect.width.min(rect.height) * 0.5)
}
