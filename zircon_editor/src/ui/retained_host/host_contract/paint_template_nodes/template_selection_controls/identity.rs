use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    TemplateComponentFamily, template_component_family,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
