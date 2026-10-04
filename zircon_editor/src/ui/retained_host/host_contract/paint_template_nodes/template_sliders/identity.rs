//! 工作台视觉语言与共享 Slider 组件家族共同决定归属；颜色与交互状态由集中选择器解析。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::super::style_selector::{select_workbench_slider_style, WorkbenchSliderStyle};

/// 组件家族识别仍受工作台视觉语言限制；普通 Slider 不应被此专用绘制器抢占。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_slider(
    node: &TemplatePaneNodeData,
) -> bool {
    uses_workbench_visual_language(node)
        && is_component_family(node, TemplateComponentFamily::Slider)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSliderStyle {
    select_workbench_slider_style(node)
}
