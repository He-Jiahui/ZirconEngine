use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_toolbar_normal_chrome_enabled(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchRun")
        || control_id.starts_with("WorkbenchLayout")
        || control_id.starts_with("WorkbenchTheme")
}
