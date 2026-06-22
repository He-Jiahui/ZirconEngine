use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::material_primitives::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_is_circular(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "circular-progress" | "spinner"
    ) || matches!(node.role.as_str(), "CircularProgress" | "Spinner")
        || component_variant_contains(node, "circular")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_is_indeterminate(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.component_role.as_str(), "spinner")
        || component_variant_contains(node, "indeterminate")
}
