use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

use super::super::super::super::paint_theme::PALETTE;
use super::super::style::{
    list_row_adornment_color, list_row_background, list_row_style, list_row_text_color,
};
use super::support::{list_node, list_node_with_flags};

#[test]
fn selected_only_list_row_uses_muted_adornment_not_selection_color() {
    let node = list_node_with_flags(true, false, false);
    let selected = list_row_style(&node);

    assert_eq!(selected.state, UiPainterResolvedState::Selected);
    assert_eq!(selected.background, Some(PALETTE.surface_pressed));
    assert_eq!(selected.text, PALETTE.text);
    assert_eq!(selected.adornment, PALETTE.text_muted);
}

#[test]
fn selected_list_row_uses_shared_surface_text_and_adornment_colors() {
    let mut node = list_node(true, false);
    node.value_color = Color::from_rgb_u8(53, 199, 208);
    node.icon_color = Color::from_rgb_u8(122, 230, 240);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(13, 65, 73, 255)));

    assert_eq!(list_row_background(&node), Some(PALETTE.surface_pressed));
    assert_eq!(list_row_text_color(&node), [53, 199, 208, 255]);
    assert_eq!(list_row_adornment_color(&node), [122, 230, 240, 255]);
}

#[test]
fn list_row_style_uses_shared_state_priority() {
    let mut node = list_node(false, true);
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    let disabled = list_row_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.border, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);

    node.disabled = false;
    let pressed = list_row_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, Some(PALETTE.surface_pressed));
    assert_eq!(pressed.border, Some(PALETTE.focus_ring));
    assert_eq!(pressed.border_width, 1.0);

    node.pressed = false;
    node.focused = false;
    node.hovered = false;
    node.selected = true;
    node.checked = true;
    let selected = list_row_style(&node);
    assert_eq!(selected.state, UiPainterResolvedState::Selected);
    assert_eq!(selected.background, Some(PALETTE.surface_pressed));
    assert_eq!(selected.text, PALETTE.text);
    assert_eq!(selected.adornment, PALETTE.focus_ring);
}
