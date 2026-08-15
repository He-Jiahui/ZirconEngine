use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_icon_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    is_component_family(node, TemplateComponentFamily::IconButton)
        && uses_workbench_visual_language(node)
        && !control_id.starts_with("WorkbenchStatus")
}
