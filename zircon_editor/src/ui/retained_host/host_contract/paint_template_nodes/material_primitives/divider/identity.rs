use super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_divider_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches_any_role(
        node.component_role.as_str(),
        node.role.as_str(),
        &["divider", "Divider"],
    ) || node.surface_variant.as_str() == "divider"
}

fn matches_any_role(component_role: &str, role: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| component_role == *candidate || role == *candidate)
}
