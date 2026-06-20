use super::super::model::WorkbenchIconButtonContext;
use super::super::palette::{WORKBENCH_ICON_PANEL_RADIUS, WORKBENCH_ICON_RAIL_RADIUS};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

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
        && node.corner_radius > WORKBENCH_ICON_PANEL_RADIUS
    {
        return node.corner_radius;
    }
    match context {
        WorkbenchIconButtonContext::Rail => WORKBENCH_ICON_RAIL_RADIUS,
        WorkbenchIconButtonContext::Toolbar | WorkbenchIconButtonContext::Panel => {
            WORKBENCH_ICON_PANEL_RADIUS
        }
    }
}
