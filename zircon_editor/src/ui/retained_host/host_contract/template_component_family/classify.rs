use super::super::data::TemplatePaneNodeData;
use super::family::TemplateComponentFamily;
use super::layout::category_layout_family;
use super::roles::{host_role_family, role_family};
use super::workbench::workbench_control_family;

pub(in crate::ui::retained_host::host_contract) fn template_component_family(
    node: &TemplatePaneNodeData,
) -> Option<TemplateComponentFamily> {
    let role = node.component_role.as_str();
    let category = node.component_category.as_str();
    let layout_role = node.component_layout_role.as_str();
    let host_role = node.role.as_str();
    let control_id = node.control_id.as_str();

    role_family(role)
        .or_else(|| host_role_family(host_role))
        .or_else(|| category_layout_family(category, layout_role))
        .or_else(|| workbench_control_family(control_id))
}

pub(in crate::ui::retained_host::host_contract) fn is_component_family(
    node: &TemplatePaneNodeData,
    family: TemplateComponentFamily,
) -> bool {
    template_component_family(node) == Some(family)
}

pub(in crate::ui::retained_host::host_contract) fn is_any_component_family(
    node: &TemplatePaneNodeData,
    families: &[TemplateComponentFamily],
) -> bool {
    template_component_family(node)
        .map(|family| families.contains(&family))
        .unwrap_or(false)
}
