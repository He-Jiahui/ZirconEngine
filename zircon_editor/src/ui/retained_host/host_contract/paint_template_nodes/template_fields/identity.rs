use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::search::is_search_field;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_field(
    node: &TemplatePaneNodeData,
) -> bool {
    (uses_workbench_visual_language(node) || is_search_field(node))
        && !node.control_id.as_str().starts_with("WorkbenchTransform")
        && is_component_family(node, TemplateComponentFamily::TextInput)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_stepper_field(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str() == "WorkbenchInputStepper"
}
