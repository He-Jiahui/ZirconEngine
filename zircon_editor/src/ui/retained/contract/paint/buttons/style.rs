//! 普通按钮接入集中样式选择器，并合成祖先传入透明度与按钮自身声明的透明度。

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    select_workbench_button_style, WorkbenchButtonKind, WorkbenchButtonStyle,
};
use super::identity::is_add_component_button;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
) -> WorkbenchButtonStyle {
    select_workbench_button_style(node, kind, is_add_component_button(node))
}

/// opacity 已包含祖先可见度；实例有限值夹到有效范围后参与合成，无效声明不污染命令。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_opacity(
    node: &TemplatePaneNodeData,
    opacity: f32,
) -> f32 {
    let declared = node.button_style.element.opacity;
    if declared.is_finite() {
        opacity * declared.clamp(0.0, 1.0)
    } else {
        opacity
    }
}
