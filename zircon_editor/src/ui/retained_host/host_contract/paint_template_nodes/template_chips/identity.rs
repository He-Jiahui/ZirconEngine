use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_chip(
    node: &TemplatePaneNodeData,
) -> bool {
    if node.control_id.as_str().starts_with("WorkbenchStatus") {
        return false;
    }
    matches!(node.control_id.as_str(), "WorkbenchChipRoot")
        || matches!(
            node.control_id.as_str(),
            "WorkbenchViewportMode"
                | "WorkbenchViewportLit"
                | "WorkbenchViewportAngle"
                | "WorkbenchViewportSpeed"
        )
        || (node.control_id.as_str().starts_with("Workbench")
            && matches!(node.component_role.as_str(), "chip" | "pill"))
}
