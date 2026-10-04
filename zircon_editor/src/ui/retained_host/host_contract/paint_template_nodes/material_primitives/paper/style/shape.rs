use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::component_variant_contains;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

/// square 变体覆盖所有圆角来源；其他 Paper 使用显式半径或宿主默认值并受当前帧限制。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    paper_corner_radius_from_host(node, rect, current_host_metrics())
}

fn paper_corner_radius_from_host(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: HostControlMetrics,
) -> f32 {
    if component_variant_contains(node, "square") {
        return 0.0;
    }
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    let radius = if configured > 0.0 {
        configured
    } else {
        metrics.radius_control
    };
    radius.min(rect.width.min(rect.height) * 0.5)
}

#[cfg(test)]
#[path = "tests/shape.rs"]
mod tests;
