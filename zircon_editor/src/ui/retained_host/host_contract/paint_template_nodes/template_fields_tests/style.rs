use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    WORKBENCH_TEXT_FIELD_DISABLED_SURFACE as FIELD_DISABLED_SURFACE,
    WORKBENCH_TEXT_FIELD_DISABLED_TEXT as FIELD_DISABLED_TEXT,
    WORKBENCH_TEXT_FIELD_FOCUSED_BORDER as FIELD_FOCUSED_BORDER,
    WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE as FIELD_FOCUSED_SURFACE,
    WORKBENCH_TEXT_FIELD_PLACEHOLDER as FIELD_PLACEHOLDER,
};
use super::super::style::{field_opacity, field_style};
use super::support::positioned_field_node;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn focused_workbench_field_uses_declared_focus_border() {
    let mut node = positioned_field_node(
        "WorkbenchInputFocused",
        "Focused input",
        12.0,
        8.0,
        170.0,
        32.0,
    );
    node.focused = true;
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(27, 152, 160, 255)));

    assert_eq!(field_border(&node), [27, 152, 160, 255]);
}

#[test]
fn disabled_workbench_field_uses_declared_opacity() {
    let mut node = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    node.disabled = true;
    node.button_style.element.opacity = 0.94;

    assert!((field_opacity(&node, 1.0) - 0.94).abs() < 0.001);
    assert!((field_opacity(&node, 0.5) - 0.47).abs() < 0.001);
}

#[test]
fn workbench_field_selector_uses_shared_text_field_state_priority() {
    let mut node =
        positioned_field_node("WorkbenchInputText", "Text field", 12.0, 8.0, 170.0, 32.0);
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Pressed);
    assert_eq!(field_surface(&node), FIELD_FOCUSED_SURFACE);

    node.pressed = false;
    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Focused);
    assert_eq!(field_border(&node), FIELD_FOCUSED_BORDER);

    node.disabled = true;
    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Disabled);
    assert_eq!(field_surface(&node), FIELD_DISABLED_SURFACE);
    assert_eq!(field_text_color(&node), FIELD_DISABLED_TEXT);

    let placeholder = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    assert_eq!(field_text_color(&placeholder), FIELD_PLACEHOLDER);
}

fn field_surface(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).surface
}

fn field_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).border
}

fn field_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).text
}

fn field_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    field_style(node).state
}
