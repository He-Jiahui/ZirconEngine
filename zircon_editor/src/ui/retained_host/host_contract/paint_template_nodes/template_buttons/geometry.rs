use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::identity::is_add_component_button;
use super::metrics::{button_geometry_metrics, button_geometry_metrics_from_host};
use crate::ui::retained_host::host_contract::paint_theme::HostControlMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    if is_add_component_button(node) {
        rect.y += add_component_button_offset_y();
    }
    rect
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn add_component_button_offset_y_from_host(
    metrics: HostControlMetrics,
) -> f32 {
    button_geometry_metrics_from_host(metrics).add_component_offset_y
}

fn add_component_button_offset_y() -> f32 {
    button_geometry_metrics().add_component_offset_y
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let metrics = button_geometry_metrics();
    let radius = if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
        node.corner_radius
    } else {
        metrics.radius
    };
    radius.min(rect.height * 0.5).max(0.0)
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
