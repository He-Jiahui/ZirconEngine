use super::super::super::data::TemplatePaneNodeData;
use super::super::template_tooltips::metrics::tooltip_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_arrow_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    let metrics = tooltip_metrics();
    let size = if node.value_number > 0.0 {
        node.value_number
    } else {
        metrics.arrow_size
    };
    size.clamp(metrics.arrow_min, metrics.arrow_max)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_icon_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    let metrics = tooltip_metrics();
    let size = if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        metrics.icon_size
    };
    size.clamp(metrics.icon_min, metrics.icon_max)
}
