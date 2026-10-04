//! 字段的透明度合成与占位样式入口；选择器接收占位判断，表面和文字共用结果。

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_text_field_style, WorkbenchTextFieldStyle};
use super::text::field_label_is_placeholder;

/// 在字段内合成继承与按钮样式的透明度；NaN 声明不会被夹值修复，最终绘制调度层会过滤该命令。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_opacity(
    node: &TemplatePaneNodeData,
    inherited_opacity: f32,
) -> f32 {
    (inherited_opacity * node.button_style.element.opacity).clamp(0.0, 1.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTextFieldStyle {
    select_workbench_text_field_style(node, field_label_is_placeholder(node))
}
