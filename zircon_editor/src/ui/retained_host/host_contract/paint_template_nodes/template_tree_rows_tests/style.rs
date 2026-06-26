use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::WORKBENCH_TREE_ROW_TEXT_SELECTED as TREE_TEXT_SELECTED;
use super::super::style::tree_row_style;
use super::support::tree_node;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn tree_row_style_uses_shared_state_priority() {
    let mut node = tree_node(
        "WorkbenchScenePropsItem",
        "TreeRow",
        "tree-row",
        "Props",
        2,
        true,
    );
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    let pressed = tree_row_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, Some(PALETTE.surface_pressed));
    assert_eq!(pressed.border, Some(PALETTE.focus_ring));
    assert_eq!(pressed.text, TREE_TEXT_SELECTED);

    node.pressed = false;
    node.selected = false;
    node.checked = false;
    let focused = tree_row_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, Some(PALETTE.surface_hover));
    assert_eq!(focused.border, Some(PALETTE.focus_ring));

    node.disabled = true;
    let disabled = tree_row_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.border, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);
}

#[test]
fn tree_row_selected_state_uses_shared_surface_without_focus_border() {
    let mut node = tree_node(
        "WorkbenchScenePlayerItem",
        "TreeRow",
        "tree-row",
        "PlayerStart",
        0,
        true,
    );
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(18, 56, 61, 255)));

    let selected = tree_row_style(&node);

    assert_eq!(selected.background, Some(PALETTE.surface_pressed));
    assert_eq!(selected.border, None);
    assert_eq!(selected.text, TREE_TEXT_SELECTED);
}
