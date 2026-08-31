use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_chip(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    if control_id.starts_with("WorkbenchStatus") {
        return false;
    }
    matches!(control_id, "WorkbenchChipRoot")
        || matches!(
            control_id,
            "WorkbenchViewportMode"
                | "WorkbenchViewportLit"
                | "WorkbenchViewportAngle"
                | "WorkbenchViewportSpeed"
        )
        || node.component_variant.as_str() == "chip"
        || (control_id.starts_with("Workbench")
            && matches!(node.component_role.as_str(), "chip" | "pill"))
}
