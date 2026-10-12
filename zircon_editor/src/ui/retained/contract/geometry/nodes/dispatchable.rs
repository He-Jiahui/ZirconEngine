use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(super) fn is_dispatchable_template_node(node: &TemplatePaneNodeData) -> bool {
    !node.disabled
        && !node.control_id.is_empty()
        && (!node.action_id.is_empty()
            || !node.binding_id.is_empty()
            || !node.dispatch_kind.is_empty()
            || !node.edit_action_id.is_empty()
            || !node.commit_action_id.is_empty()
            || matches!(node.component_role.as_str(), "input-field" | "number-field"))
}
