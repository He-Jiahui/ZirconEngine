use super::palette::workbench_tree_row_palette_from_host;
use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn tree_row_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.selected = true;
    node.checked = true;
    node.button_style.loading = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.background, None);
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.icon, PALETTE.text_disabled);
    assert_eq!(style.secondary, PALETTE.text_disabled);
    assert_eq!(style.action, PALETTE.text_disabled);
}

#[test]
fn selected_tree_row_uses_muted_selected_fill_and_neutral_outline() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;
    node.checked = true;
    node.focused = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface_pressed));
    assert_ne!(style.background, Some(PALETTE.surface_selected));
    assert_eq!(style.border, Some(PALETTE.border));
    assert_ne!(style.border, Some(PALETTE.accent));
    assert_ne!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.border_width, 1.0);
}

#[test]
fn focused_unmarked_tree_row_keeps_keyboard_focus_border_without_hover_fill() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, None);
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.border_width, 1.0);
}

#[test]
fn hovered_unmarked_tree_row_still_uses_hover_fill_without_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Hovered);
    assert_eq!(style.background, Some(PALETTE.surface_hover));
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
}

#[test]
fn tree_row_palette_projects_surface_text_and_focus_roles_from_host_palette() {
    let mut host = PALETTE;
    host.surface_pressed = [11, 12, 13, 255];
    host.surface_hover = [21, 22, 23, 255];
    host.focus_ring = [31, 32, 33, 255];
    host.text = [41, 42, 43, 255];
    host.text_muted = [51, 52, 53, 255];
    host.text_disabled = [61, 62, 63, 255];

    let palette = workbench_tree_row_palette_from_host(host);

    assert_eq!(palette.marked_surface, [11, 12, 13, 255]);
    assert_eq!(palette.hot_surface, [21, 22, 23, 255]);
    assert_eq!(palette.focus_border, [31, 32, 33, 255]);
    assert_eq!(palette.text, [41, 42, 43, 255]);
    assert_eq!(palette.text_muted, [51, 52, 53, 255]);
    assert_eq!(palette.text_disabled, [61, 62, 63, 255]);
}
