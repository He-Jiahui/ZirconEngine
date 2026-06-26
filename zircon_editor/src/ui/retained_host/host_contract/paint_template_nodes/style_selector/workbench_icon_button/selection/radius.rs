use super::super::model::WorkbenchIconButtonContext;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_radius(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
) -> f32 {
    let declared = node.button_style.element.corner_radius;
    if declared.is_finite() && declared > 0.0 {
        return declared;
    }
    if context == WorkbenchIconButtonContext::Panel
        && node.corner_radius.is_finite()
        && node.corner_radius > METRICS.radius_control
    {
        return node.corner_radius;
    }
    match context {
        WorkbenchIconButtonContext::Rail => METRICS.radius_control,
        WorkbenchIconButtonContext::Toolbar | WorkbenchIconButtonContext::Panel => {
            METRICS.radius_control
        }
    }
}
