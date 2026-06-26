use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    WORKBENCH_DROPDOWN_FOCUS_BORDER as DROPDOWN_FOCUS_BORDER,
    WORKBENCH_DROPDOWN_OPEN_SURFACE as DROPDOWN_OPEN_SURFACE,
};
use super::super::style::dropdown_style;
use super::support::{dropdown_node, resolved_background_and_border};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn workbench_dropdown_uses_declared_style_text_and_chevron_colors() {
    let mut node = dropdown_node(false);
    node.button_style = resolved_background_and_border([32, 38, 42, 255], [31, 39, 46, 255]);
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(127, 138, 145);
    node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(103, 115, 122);

    assert_eq!(dropdown_surface(&node), [32, 38, 42, 255]);
    assert_eq!(dropdown_border(&node), [31, 39, 46, 255]);
    assert_eq!(dropdown_text_color(&node), [127, 138, 145, 255]);
    assert_eq!(dropdown_chevron_color(&node), [103, 115, 122, 255]);
}

#[test]
fn workbench_dropdown_selector_uses_shared_state_priority() {
    let pressed_open = TemplatePaneNodeData {
        popup_open: true,
        focused: true,
        pressed: true,
        ..dropdown_node(false)
    };
    let disabled_pressed = TemplatePaneNodeData {
        disabled: true,
        pressed: true,
        ..dropdown_node(false)
    };

    assert_eq!(
        dropdown_visual_state(&pressed_open),
        UiPainterResolvedState::Pressed
    );
    assert_eq!(dropdown_surface(&pressed_open), DROPDOWN_OPEN_SURFACE);
    assert_eq!(dropdown_border(&pressed_open), DROPDOWN_FOCUS_BORDER);
    assert_eq!(
        dropdown_visual_state(&disabled_pressed),
        UiPainterResolvedState::Disabled
    );
}

fn dropdown_surface(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).surface
}

fn dropdown_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).border
}

fn dropdown_chevron_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).chevron
}

fn dropdown_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).text
}

fn dropdown_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    dropdown_style(node).state
}
