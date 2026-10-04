//! 以控件身份确定工具栏、侧轨或面板语境，随后交给集中选择器处理交互与主题；页签关闭按钮归工具栏语境。

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    select_workbench_icon_button_style, WorkbenchIconButtonContext, WorkbenchIconButtonStyle,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) type IconButtonContext =
    WorkbenchIconButtonContext;

/// 控件身份决定密度语境；页签关闭按钮沿用工具栏尺寸，新增身份需与布局端契约一同核对。
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_style(
    node: &TemplatePaneNodeData,
    context: IconButtonContext,
) -> WorkbenchIconButtonStyle {
    select_workbench_icon_button_style(node, context)
}

#[cfg(test)]
#[path = "tests/style_optimization_tests.rs"]
mod optimization_tests;
