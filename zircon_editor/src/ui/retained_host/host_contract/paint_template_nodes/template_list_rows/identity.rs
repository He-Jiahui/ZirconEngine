use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, TemplateComponentFamily,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_list_row(
    node: &TemplatePaneNodeData,
) -> bool {
    is_component_family(node, TemplateComponentFamily::ListRow)
        && !node.control_id.as_str().ends_with("Title")
}
