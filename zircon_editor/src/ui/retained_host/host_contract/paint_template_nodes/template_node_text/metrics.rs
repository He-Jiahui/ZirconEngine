use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TEXT_HORIZONTAL_INSET: f32 = 4.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TEXT_VERTICAL_INSET:
    f32 = 4.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MIN_TEXT_RECT_HEIGHT:
    f32 = 12.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn node_font_size(
    node: &TemplatePaneNodeData,
    available_height: f32,
) -> f32 {
    node_font_size_from_host(node, available_height, current_host_metrics())
}

fn node_font_size_from_host(
    node: &TemplatePaneNodeData,
    available_height: f32,
    metrics: HostControlMetrics,
) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        metrics.font_body
    };
    requested.min(available_height.max(1.0)).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn template_node_default_font_size_projects_from_host_typography() {
        let node = TemplatePaneNodeData::default();
        let metrics = HostControlMetrics {
            font_body: 15.0,
            ..METRICS
        };

        assert_eq!(node_font_size_from_host(&node, 20.0, metrics), 15.0);
        assert_eq!(node_font_size_from_host(&node, 10.0, metrics), 10.0);
    }
}
