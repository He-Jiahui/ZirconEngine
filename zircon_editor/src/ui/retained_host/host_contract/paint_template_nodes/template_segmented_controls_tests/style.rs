use super::super::super::super::paint_theme::PALETTE;
use super::super::style::{
    SEGMENT_SELECTED_BACKGROUND, segmented_control_style, selected_segment_border_width,
    selected_segment_surface_color, selected_segment_underline_color,
    selected_segment_underline_height, tab_style, tab_text_color,
};
use super::support::{segmented_node, tab_node};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn selected_segment_style_defaults_to_underlined_slate_tab_without_declaration() {
    let node = segmented_node();

    assert_eq!(selected_segment_border_width(&node), 0.0);
    assert_eq!(
        selected_segment_surface_color(&node),
        SEGMENT_SELECTED_BACKGROUND
    );
    assert_eq!(
        selected_segment_surface_color(&node),
        PALETTE.surface_pressed
    );
    assert_ne!(
        selected_segment_surface_color(&node),
        PALETTE.surface_selected
    );
    assert_eq!(selected_segment_underline_height(&node), 2.0);
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
    assert_eq!(style.state, UiPainterResolvedState::Selected);
    assert_eq!(style.background, None);
    assert_eq!(tab_text_color(&tab), PALETTE.text);
}

#[test]
fn focused_segmented_control_keeps_idle_background_with_focus_border() {
    let mut node = segmented_node();
    node.focused = true;

    let style = segmented_control_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface));
    assert_eq!(style.border, Some(PALETTE.focus_ring));
}

#[test]
fn focused_hovered_segmented_control_keeps_hover_background_and_focus_border() {
    let mut node = segmented_node();
    node.focused = true;
    node.hovered = true;

    let style = segmented_control_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface_hover));
    assert_eq!(style.border, Some(PALETTE.focus_ring));
}

#[test]
fn focused_tab_keeps_normal_background_without_active_border() {
    let mut node = tab_node();
    node.checked = false;
    node.selected = false;
    node.focused = true;

    let style = tab_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, None);
    assert_eq!(style.border, None);
}
