use super::*;
use zircon_runtime_interface::ui::style::{
    ButtonInteractionState, ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

#[test]
fn native_template_button_state_uses_shared_painter_priority() {
    let mut node = button_node();
    node.focused = true;
    node.pressed = true;

    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Pressed
    );
    assert_eq!(surface_color(&node), PALETTE.surface_pressed);

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
    assert_eq!(surface_color(&node), PALETTE.surface_disabled);
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

fn button_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "Button".into(),
        control_id: "TemplateStyleButton".into(),
        ..TemplatePaneNodeData::default()
    }
}

fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                color[0], color[1], color[2], color[3],
            ))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}
