use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::resolved_state_for_node;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_button_disabled(
    node: &TemplatePaneNodeData,
) -> bool {
    node.disabled
        || node.button_style.disabled
        || matches!(
            node.button_style.interaction_state,
            ButtonInteractionState::Disabled
        )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_interaction_state(
    node: &TemplatePaneNodeData,
) -> ButtonInteractionState {
    resolved_state_for_node(node).button_interaction_state()
}
