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
        // Runtime focus modality is authoritative once projected. Legacy/static previews retain
        // their authored focus appearance until a live pointer or keyboard cause is known.
        focus_visible: focus_visible_for_node(node),
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn focus_visible_for_node(
    node: &TemplatePaneNodeData,
) -> bool {
    if node.focus_visible_known {
        node.focus_visible
    } else {
        node.focus_visible
            || node.focused
            || matches!(
                node.button_style.interaction_state,
                ButtonInteractionState::Focused
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_pointer_focus_remains_semantic_without_drawing_keyboard_focus() {
        let node = TemplatePaneNodeData {
            focused: true,
            focus_visible: false,
            focus_visible_known: true,
            ..TemplatePaneNodeData::default()
        };

        let state = resolved_state_for_node(&node);

        assert!(state.focused);
        assert!(!state.focus_visible);
    }

    #[test]
    fn runtime_keyboard_focus_and_static_preview_keep_visible_focus() {
        let keyboard = TemplatePaneNodeData {
            focused: true,
            focus_visible: true,
            focus_visible_known: true,
            ..TemplatePaneNodeData::default()
        };
        let static_preview = TemplatePaneNodeData {
            focus_visible: true,
            ..TemplatePaneNodeData::default()
        };

        assert!(resolved_state_for_node(&keyboard).focus_visible);
        assert!(resolved_state_for_node(&static_preview).focus_visible);
    }
}
