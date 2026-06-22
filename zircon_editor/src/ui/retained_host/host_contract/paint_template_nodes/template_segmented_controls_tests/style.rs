use super::super::super::super::paint_theme::PALETTE;
use super::super::style::{
    segmented_control_style, selected_segment_border_width, selected_segment_underline_color,
    selected_segment_underline_height, tab_style, tab_text_color,
};
use super::support::{segmented_node, tab_node};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn selected_segment_style_defaults_to_legacy_border_without_declaration() {
    let node = segmented_node();

    assert_eq!(selected_segment_border_width(&node), 1.0);
    assert_eq!(selected_segment_underline_height(&node), 0.0);
    assert_eq!(selected_segment_underline_color(&node), PALETTE.accent);
}

#[test]
fn selected_segment_style_honors_declared_border_suppression_and_underline() {
    let mut node = segmented_node();
    node.has_selected_segment_border_width = true;
    node.selected_segment_border_width = 0.0;
    node.selected_segment_underline_height = 1.0;
    node.selected_segment_underline_color = Color::from_argb_u8(122, 50, 211, 222);

    assert_eq!(selected_segment_border_width(&node), 0.0);
    assert_eq!(selected_segment_underline_height(&node), 1.0);
    assert_eq!(selected_segment_underline_color(&node), [50, 211, 222, 122]);
}

#[test]
fn segmented_and_tab_styles_use_shared_state_priority() {
    let mut node = segmented_node();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let segmented = segmented_control_style(&node);
    assert_eq!(segmented.background, Some(PALETTE.surface_disabled));
    assert_eq!(segmented.border, Some(PALETTE.border_disabled));
    assert_eq!(segmented.selected_text, PALETTE.text_disabled);

    node.disabled = false;
    let segmented = segmented_control_style(&node);
    assert_eq!(segmented.state, UiPainterResolvedState::Pressed);
    assert_eq!(segmented.background, Some(PALETTE.surface_pressed));
    assert_eq!(segmented.border, Some(PALETTE.accent));

    let mut tab = tab_node();
    tab.checked = true;
    tab.hovered = true;
    let style = tab_style(&tab);
    assert_eq!(style.state, UiPainterResolvedState::Hovered);
    assert_eq!(style.background, Some(PALETTE.surface_hover));
    assert_eq!(tab_text_color(&tab), PALETTE.text);
}
