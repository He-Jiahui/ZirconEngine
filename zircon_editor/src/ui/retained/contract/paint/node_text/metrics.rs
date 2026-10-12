use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{
    current_host_metrics, logical_font_size_to_physical, HostControlMetrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TemplateNodeTextGeometryMetrics {
    pub(super) horizontal_inset: f32,
    pub(super) vertical_inset: f32,
    pub(super) minimum_text_height: f32,
    pub(super) edge_guard: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn node_font_size(
    node: &TemplatePaneNodeData,
    available_height: f32,
) -> f32 {
    node_font_size_from_host(node, available_height, current_host_metrics())
}

pub(super) fn template_node_text_geometry_metrics() -> TemplateNodeTextGeometryMetrics {
    template_node_text_geometry_metrics_from_host(current_host_metrics())
}

pub(super) fn template_node_text_line_height(font_size: f32) -> f32 {
    if !font_size.is_finite() || font_size <= 0.0 {
        return 0.0;
    }
    template_node_text_line_height_from_host(font_size, current_host_metrics())
}

fn template_node_text_line_height_from_host(font_size: f32, metrics: HostControlMetrics) -> f32 {
    if !font_size.is_finite() || font_size <= 0.0 {
        return 0.0;
    }
    let line_height = metrics.line_height(font_size).max(font_size);
    if line_height.is_finite() && line_height > 0.0 {
        line_height
    } else {
        font_size
    }
}

fn node_font_size_from_host(
    node: &TemplatePaneNodeData,
    available_height: f32,
    metrics: HostControlMetrics,
) -> f32 {
    // Authored node sizes are logical. Host role defaults have already been
    // projected to physical pixels with the rest of HostControlMetrics.
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        logical_font_size_to_physical(node.font_size, metrics.scale_factor)
    } else if node.role.as_str() == "Label"
        && matches!(node.text_tone.as_str(), "muted" | "subtle" | "secondary")
    {
        metrics.font_small
    } else {
        metrics.font_body
    };
    if !available_height.is_finite() || available_height <= 0.0 {
        return 0.0;
    }
    if requested.is_finite() && requested > 0.0 {
        requested
    } else {
        0.0
    }
}

fn template_node_text_geometry_metrics_from_host(
    metrics: HostControlMetrics,
) -> TemplateNodeTextGeometryMetrics {
    TemplateNodeTextGeometryMetrics {
        horizontal_inset: metrics.gap_s,
        vertical_inset: metrics.gap_s,
        minimum_text_height: metrics
            .line_height(metrics.font_small)
            .round()
            .max(metrics.font_small.ceil()),
        edge_guard: metrics.border_width,
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
