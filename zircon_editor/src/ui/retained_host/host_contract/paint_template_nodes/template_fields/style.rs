use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{WorkbenchTextFieldStyle, select_workbench_text_field_style};
use super::text::field_label_is_placeholder;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_opacity(
    node: &TemplatePaneNodeData,
    inherited_opacity: f32,
) -> f32 {
    (inherited_opacity * node.button_style.element.opacity).clamp(0.0, 1.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTextFieldStyle {
    select_workbench_text_field_style(node, field_label_is_placeholder(node))
}
