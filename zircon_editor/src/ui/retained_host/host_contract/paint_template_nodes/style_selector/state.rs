use super::super::super::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{ButtonInteractionState, UiPainterState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolved_state_for_node(
    node: &TemplatePaneNodeData,
) -> UiPainterState {
    let style_state = node.button_style.interaction_state;
    UiPainterState {
        hovered: node.hovered || matches!(style_state, ButtonInteractionState::Hover),
        pressed: node.pressed
            || node.enter_pressed
            || matches!(style_state, ButtonInteractionState::Pressed),
        focused: node.focused || matches!(style_state, ButtonInteractionState::Focused),
        // The committed editor node contract does not yet carry input modality. M4 replaces
        // this legacy fallback with the retained runtime focus-visible projection.
        focus_visible: node.focused || matches!(style_state, ButtonInteractionState::Focused),
        disabled: node.disabled
            || node.button_style.disabled
            || matches!(style_state, ButtonInteractionState::Disabled),
        checked: node.checked,
        selected: node.selected,
        open: node.popup_open,
        dragging: node.dragging,
        drop_hovered: node.drop_hovered || node.active_drag_target,
        loading: node.button_style.loading
            || matches!(style_state, ButtonInteractionState::Loading),
    }
}
