//! 把共享组件家族识别投影到树行绘制，防止场景 ID 与显式 role 各自建立不同识别规则。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, TemplateComponentFamily,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_tree_row(
    node: &TemplatePaneNodeData,
) -> bool {
    is_component_family(node, TemplateComponentFamily::TreeRow)
}
