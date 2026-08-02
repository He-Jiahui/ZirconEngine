use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::template_component_family::{
    TemplateComponentFamily, template_component_family,
};

pub(super) fn is_dispatchable(node: &TemplatePaneNodeData) -> bool {
    let family = template_component_family(node);
    !node.disabled
        && !node.control_id.is_empty()
        && (!node.action_id.is_empty()
            || !node.binding_id.is_empty()
            || !node.dispatch_kind.is_empty()
            || !node.edit_action_id.is_empty()
            || !node.commit_action_id.is_empty()
            || family == Some(TemplateComponentFamily::TextInput))
}

pub(super) fn template_component(node: &TemplatePaneNodeData) -> String {
    if node.component_role.is_empty() {
        template_component_family(node)
            .map(TemplateComponentFamily::as_str)
            .unwrap_or_default()
            .to_string()
    } else {
        node.component_role.to_string()
    }
}
