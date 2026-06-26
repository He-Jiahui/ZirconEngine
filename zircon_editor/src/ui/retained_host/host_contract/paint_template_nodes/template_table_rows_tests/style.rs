use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::WORKBENCH_TABLE_HOVER_BG as TABLE_HOVER_BG;
use super::super::style::table_row_style;
use super::support::table_node;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn workbench_table_row_style_uses_shared_state_priority() {
    let mut node = table_node("WorkbenchTableRowRoot", false);
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    let pressed = table_row_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, PALETTE.surface_pressed);
    assert_eq!(pressed.border, Some(PALETTE.focus_ring));
    assert_eq!(pressed.text_for_cell(0), PALETTE.text);

    node.pressed = false;
    let focused = table_row_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, TABLE_HOVER_BG);
    assert_eq!(focused.border, Some(PALETTE.focus_ring));

    node.disabled = true;
    let disabled = table_row_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, PALETTE.surface_disabled);
    assert_eq!(disabled.border, None);
    assert_eq!(disabled.text_for_cell(0), PALETTE.text_disabled);
}

#[test]
fn selected_table_row_uses_neutral_surface_even_with_declared_background() {
    let mut node = table_node("WorkbenchTableSelected", true);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(23, 57, 66, 255)));

    let style = table_row_style(&node);

    assert_eq!(style.background, PALETTE.surface_pressed);
    assert_ne!(style.background, PALETTE.surface_selected);
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
}
