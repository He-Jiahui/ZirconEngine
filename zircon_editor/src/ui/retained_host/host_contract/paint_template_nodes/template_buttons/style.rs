use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    WorkbenchButtonKind, WorkbenchButtonStyle, select_workbench_button_style,
};
use super::identity::is_add_component_button;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
) -> WorkbenchButtonStyle {
    select_workbench_button_style(node, kind, is_add_component_button(node))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_opacity(
    node: &TemplatePaneNodeData,
    opacity: f32,
) -> f32 {
    let declared = node.button_style.element.opacity;
    if declared.is_finite() {
        opacity * declared.clamp(0.0, 1.0)
    } else {
        opacity
    }
}
