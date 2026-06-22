use super::super::state::button_interaction_state;
use super::support::button_node;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

#[test]
fn native_template_button_state_uses_shared_painter_priority() {
    let mut node = button_node();
    node.focused = true;
    node.pressed = true;

    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Pressed
    );

    node.button_style.loading = true;
    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Loading
    );

    node.disabled = true;
    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Disabled
    );
}

#[test]
fn native_template_button_style_state_values_feed_shared_priority() {
    let mut node = button_node();
    node.button_style.interaction_state = ButtonInteractionState::Pressed;
    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Pressed
    );

    node.button_style.interaction_state = ButtonInteractionState::Loading;
    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Loading
    );

    node.button_style.interaction_state = ButtonInteractionState::Disabled;
    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Disabled
    );
}
