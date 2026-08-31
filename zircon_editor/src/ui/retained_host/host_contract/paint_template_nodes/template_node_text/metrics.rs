use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

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
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn template_node_font_role_projects_caption_and_body_from_host_typography() {
        let metrics = HostControlMetrics {
            font_small: 11.0,
            font_body: 15.0,
            ..METRICS
        };
        let body = TemplatePaneNodeData::default();
        let mut caption = TemplatePaneNodeData::default();
        caption.role = "Label".into();
        caption.text_tone = "muted".into();
        let mut authored = caption.clone();
        authored.font_size = 12.0;
        let mut subtle = caption.clone();
        subtle.text_tone = "subtle".into();
        let mut secondary = caption.clone();
        secondary.text_tone = "secondary".into();

        assert_eq!(node_font_size_from_host(&body, 20.0, metrics), 15.0);
        assert_eq!(node_font_size_from_host(&body, 10.0, metrics), 15.0);
        assert_eq!(node_font_size_from_host(&body, 0.0, metrics), 0.0);
        assert_eq!(node_font_size_from_host(&caption, 20.0, metrics), 11.0);
        assert_eq!(node_font_size_from_host(&caption, 6.0, metrics), 11.0);
        assert_eq!(node_font_size_from_host(&subtle, 20.0, metrics), 11.0);
        assert_eq!(node_font_size_from_host(&secondary, 20.0, metrics), 11.0);
        assert_eq!(node_font_size_from_host(&authored, 20.0, metrics), 12.0);
        assert_eq!(node_font_size_from_host(&authored, 6.0, metrics), 12.0);
    }

    #[test]
    fn template_node_text_geometry_projects_shared_gap_border_and_caption_line_height() {
        let metrics = HostControlMetrics {
            gap_s: 5.0,
            border_width: 1.5,
            font_small: 10.0,
            line_height_ratio: 1.3,
            ..METRICS
        };

        let geometry = template_node_text_geometry_metrics_from_host(metrics);

        assert_eq!(geometry.horizontal_inset, 5.0);
        assert_eq!(geometry.vertical_inset, 5.0);
        assert_eq!(geometry.minimum_text_height, 13.0);
        assert_eq!(geometry.edge_guard, 1.5);
    }

    #[test]
    fn template_node_text_line_height_keeps_the_runtime_fractional_metric() {
        let metrics = HostControlMetrics {
            line_height_ratio: 1.35,
            ..METRICS
        };

        assert!((template_node_text_line_height_from_host(11.0, metrics) - 14.85).abs() < 0.001);
    }
}
