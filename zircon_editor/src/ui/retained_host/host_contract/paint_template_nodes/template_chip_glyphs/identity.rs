use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_has_chevron(
    node: &TemplatePaneNodeData,
) -> bool {
    node.popup_open
        || node.options.row_count() > 0
        || matches!(
            node.control_id.as_str(),
            "WorkbenchViewportMode" | "WorkbenchViewportAngle" | "WorkbenchViewportSpeed"
        )
}
