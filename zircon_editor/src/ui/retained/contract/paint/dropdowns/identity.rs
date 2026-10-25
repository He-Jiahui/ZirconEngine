//! 下拉组件语义进入专用绘制链；旧工作台视觉声明继续兼容，共享家族阻止冲突身份误入。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_dropdown(
    node: &TemplatePaneNodeData,
) -> bool {
    is_component_family(node, TemplateComponentFamily::Dropdown)
        && (matches!(
            node.component_role.as_str(),
            "dropdown" | "combo-box" | "select" | "search-select"
        ) || matches!(node.role.as_str(), "Dropdown" | "ComboBox" | "Select")
            || uses_workbench_visual_language(node))
}
