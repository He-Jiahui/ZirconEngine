use super::super::super::super::paint_theme::PALETTE;
use super::super::colors::surface_color;
use super::super::state::button_interaction_state;
use super::support::{button_node, resolved_background};
use zircon_runtime_interface::ui::style::ButtonInteractionState;

#[test]
fn native_template_button_state_resolves_shared_surface_priority() {
    let mut node = button_node();
    node.focused = true;
    node.pressed = true;

    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Pressed
    );
    assert_eq!(surface_color(&node), PALETTE.surface_pressed);

    node.button_style.loading = true;
    node.disabled = true;
    assert_eq!(surface_color(&node), PALETTE.surface_disabled);
}

#[test]
fn native_template_button_style_keeps_declared_colors_after_state_resolution() {
    let mut node = button_node();
    node.hovered = true;
    node.button_style = resolved_background([11, 22, 33, 255]);

    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Hover
    );
    assert_eq!(surface_color(&node), PALETTE.surface_hover);

    node.hovered = false;
    assert_eq!(surface_color(&node), [11, 22, 33, 255]);
}
