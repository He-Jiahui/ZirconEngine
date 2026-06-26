use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    select_workbench_icon_button_style, WorkbenchIconButtonContext, WorkbenchIconButtonStyle,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) type IconButtonContext =
    WorkbenchIconButtonContext;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_context(
    node: &TemplatePaneNodeData,
) -> IconButtonContext {
    let control_id = node.control_id.as_str();
    if control_id.starts_with("WorkbenchRail") {
        IconButtonContext::Rail
    } else if is_tab_close_button(control_id) {
        IconButtonContext::Toolbar
    } else if control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchRun")
        || control_id.starts_with("WorkbenchLayout")
        || control_id.starts_with("WorkbenchTheme")
    {
        IconButtonContext::Toolbar
    } else {
        IconButtonContext::Panel
    }
}

fn is_tab_close_button(control_id: &str) -> bool {
    control_id.starts_with("DockTabClose")
        || control_id.starts_with("PageTabClose")
        || control_id.starts_with("DocumentTabClose")
        || control_id.ends_with("TabClose")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_style(
    node: &TemplatePaneNodeData,
    context: IconButtonContext,
) -> WorkbenchIconButtonStyle {
    select_workbench_icon_button_style(node, context)
}
