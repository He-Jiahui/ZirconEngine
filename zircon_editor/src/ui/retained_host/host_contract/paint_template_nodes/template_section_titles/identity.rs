use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_section_title(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.control_id.as_str(),
        "WorkbenchSectionTitleRoot" | "WorkbenchTransformLabel" | "WorkbenchMeshLabel"
    ) || (node.control_id.as_str().starts_with("Workbench")
        && node.control_id.as_str().ends_with("Title"))
}
