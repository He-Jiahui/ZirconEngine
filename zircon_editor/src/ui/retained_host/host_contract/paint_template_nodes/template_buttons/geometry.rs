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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_button_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    inner: &FrameRect,
    outer: &FrameRect,
) -> bool {
    if !has_paintable_button_extent(inner) || !has_paintable_button_extent(outer) {
        return false;
    }

    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let outer_right = outer.x + outer.width;
    let outer_bottom = outer.y + outer.height;
    inner_right.is_finite()
        && inner_bottom.is_finite()
        && outer_right.is_finite()
        && outer_bottom.is_finite()
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner_right <= outer_right
        && inner_bottom <= outer_bottom
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    if !has_paintable_button_extent(rect) {
        return FrameRect {
            x: rect.x,
            y: rect.y,
            width: 0.0,
            height: 0.0,
        };
    }

    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    if !right.is_finite() || !bottom.is_finite() {
        return FrameRect {
            x: rect.x,
            y: rect.y,
            width: 0.0,
            height: 0.0,
        };
    }

    let x = rect.x.ceil();
    let y = rect.y.ceil();
    let right = right.floor();
    let bottom = bottom.floor();
    FrameRect {
        x,
        y,
        width: (right - x).max(0.0),
        height: (bottom - y).max(0.0),
    }
}
