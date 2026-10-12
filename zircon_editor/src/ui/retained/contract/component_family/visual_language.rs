use super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract) fn uses_workbench_visual_language(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str().starts_with("Workbench")
        || node.component_variant.as_str().contains("workbench")
        || node.surface_variant.as_str().contains("workbench")
        || node.button_variant.as_str().contains("workbench")
}
