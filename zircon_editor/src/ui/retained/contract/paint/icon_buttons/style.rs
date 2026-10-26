//! 以控件身份确定密度语境，页签关闭按钮使用 Starship 的独立无底板配方。

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    select_workbench_icon_button_style, WorkbenchIconButtonContext, WorkbenchIconButtonStyle,
};
use super::identity::is_tab_close_button;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) type IconButtonContext =
    WorkbenchIconButtonContext;

/// 控件身份决定密度语境；页签关闭按钮使用 16px 图形和独立状态配方。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_context(
    node: &TemplatePaneNodeData,
) -> IconButtonContext {
    let control_id = node.control_id.as_str();
    if is_tab_close_button(control_id) {
        IconButtonContext::TabClose
    } else if control_id.starts_with("WorkbenchRail") {
        IconButtonContext::Rail
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_style(
    node: &TemplatePaneNodeData,
    context: IconButtonContext,
) -> WorkbenchIconButtonStyle {
    select_workbench_icon_button_style(node, context)
}

#[cfg(test)]
#[path = "tests/style_perf.rs"]
mod optimization_tests;
