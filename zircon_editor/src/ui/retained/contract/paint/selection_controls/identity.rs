//! 共享组件家族映射到复选、单选或开关；三者同属 primary 绘制位置，避免归属优先级互相冲突。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    template_component_family, TemplateComponentFamily,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// primary 绘制链的三种可独占选择控件；不要与一般 segmented 选项混用。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum SelectionControlKind {
    Checkbox,
    Radio,
    Toggle,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_control_kind(
    node: &TemplatePaneNodeData,
) -> Option<SelectionControlKind> {
    match template_component_family(node) {
        Some(TemplateComponentFamily::Checkbox) => Some(SelectionControlKind::Checkbox),
        Some(TemplateComponentFamily::Radio) => Some(SelectionControlKind::Radio),
        Some(TemplateComponentFamily::Toggle) => Some(SelectionControlKind::Toggle),
        _ => None,
    }
}
