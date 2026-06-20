use super::super::super::data::TemplatePaneNodeData;
use super::state::button_interaction_state;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    let width = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if matches!(
        button_interaction_state(node),
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused
    ) || node.selected
        || node.checked
    {
        width.max(2.0)
    } else {
        width
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_corner_radius(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}
