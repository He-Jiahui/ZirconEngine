use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_dropdown(
    node: &TemplatePaneNodeData,
) -> bool {
    uses_workbench_visual_language(node)
        && is_component_family(node, TemplateComponentFamily::Dropdown)
}
