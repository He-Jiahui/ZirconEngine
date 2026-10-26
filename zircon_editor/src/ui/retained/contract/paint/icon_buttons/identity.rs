//! 通过共享组件家族接管工作台和页签关闭图标，显式让出 WorkbenchStatus 给状态控件绘制器。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_icon_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    is_component_family(node, TemplateComponentFamily::IconButton)
        && (uses_workbench_visual_language(node) || is_tab_close_button(control_id))
        && !control_id.starts_with("WorkbenchStatus")
}

pub(super) fn is_tab_close_button(control_id: &str) -> bool {
    if control_id.ends_with("TabClose") {
        return true;
    }
    match control_id.as_bytes().first() {
        Some(b'D') => {
            control_id.starts_with("DockTabClose") || control_id.starts_with("DocumentTabClose")
        }
        Some(b'P') => control_id.starts_with("PageTabClose"),
        _ => false,
    }
}
