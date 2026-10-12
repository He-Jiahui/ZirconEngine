//! 结合弹出状态、选项和固定视口控件身份判断 chip 是否承载可展开语义。

use super::super::super::data::TemplatePaneNodeData;

/// 弹出态或有选项代表展开提示；固定视口模式、角度和速度控件即使尚无选项也保留入口箭头。
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
