//! 字段归属结合共享组件家族和搜索身份；变换字段走更专用的属性/轴值绘制器。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::search::is_search_field;

/// secondary 链按共享文本输入组件家族接管；搜索身份可使非标准 Workbench 名称归入字段，但变换输入被排除。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_field(
    node: &TemplatePaneNodeData,
) -> bool {
    (uses_workbench_visual_language(node) || is_search_field(node))
        && !node.control_id.as_str().starts_with("WorkbenchTransform")
        && is_component_family(node, TemplateComponentFamily::TextInput)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_stepper_field(
    node: &TemplatePaneNodeData,
) -> bool {
    node.layout_stepper || node.control_id.as_str() == "WorkbenchInputStepper"
}
