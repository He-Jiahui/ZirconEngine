use super::super::model::WorkbenchIconButtonContext;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostControlMetrics, current_host_metrics,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_radius(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
) -> f32 {
    icon_radius_from_host(node, context, current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_radius_from_host(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    metrics: HostControlMetrics,
) -> f32 {
    let declared = node.button_style.element.corner_radius;
    if declared.is_finite() && declared > 0.0 {
        return declared;
    }
    if context == WorkbenchIconButtonContext::Panel
        && node.corner_radius.is_finite()
        && node.corner_radius > metrics.radius_control
    {
        return node.corner_radius;
    }
    match context {
        WorkbenchIconButtonContext::Rail => metrics.radius_control,
        WorkbenchIconButtonContext::Toolbar | WorkbenchIconButtonContext::Panel => {
            metrics.radius_control
        }
    }
}
