//! 共享组件家族是分段组与页签的归属来源；控制 ID 不再在本域单独猜测。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, TemplateComponentFamily,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_segmented_control(
    node: &TemplatePaneNodeData,
) -> bool {
    is_component_family(node, TemplateComponentFamily::SegmentedControl)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_tab(
    node: &TemplatePaneNodeData,
) -> bool {
    is_component_family(node, TemplateComponentFamily::Tab)
}
