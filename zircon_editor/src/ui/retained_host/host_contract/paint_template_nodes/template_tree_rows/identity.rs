use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    TemplateComponentFamily, is_component_family,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_tree_row(
    node: &TemplatePaneNodeData,
) -> bool {
    is_component_family(node, TemplateComponentFamily::TreeRow)
}
