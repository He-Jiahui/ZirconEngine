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
fn selected_tree_row_preserves_primary_fill_and_independent_keyboard_outline() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;
    node.checked = true;
    node.focused = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface_selected));
    assert_ne!(style.background, Some(PALETTE.surface_pressed));
    assert_eq!(style.border, Some(PALETTE.accent));
    assert_ne!(style.border, Some(PALETTE.border));
    assert_eq!(style.border_width, 1.0);
}

#[test]
fn selected_hovered_tree_row_keeps_selection_priority_over_hover_fill() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;
    node.hovered = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.background, Some(PALETTE.surface_selected));
    assert_ne!(style.background, Some(PALETTE.accent_soft));
    assert_ne!(style.background, Some(PALETTE.surface_pressed));
}

#[test]
fn pressed_unmarked_tree_row_keeps_pressed_surface_distinct_from_selection() {
    let mut node = TemplatePaneNodeData::default();
    node.pressed = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.background, Some(PALETTE.surface_pressed));
    assert_ne!(style.background, Some(PALETTE.surface_selected));
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
}

#[test]
fn pressed_selected_tree_row_preserves_selection_surface() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;
    node.pressed = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.background, Some(PALETTE.surface_selected));
    assert_ne!(style.background, Some(PALETTE.surface_pressed));
    assert_ne!(style.background, Some(PALETTE.accent_soft));
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
    assert_eq!(style.background, Some(PALETTE.surface_panel));
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
}

#[test]
fn tree_row_palette_projects_surface_text_and_focus_roles_from_host_palette() {
    let mut host = PALETTE;
    host.surface_pressed = [6, 7, 8, 255];
    host.surface_selected = [9, 10, 11, 255];
    host.surface_selected_inactive = [12, 13, 14, 255];
    host.text_selected = [15, 16, 17, 255];
    host.surface_panel = [21, 22, 23, 255];
    host.focus_ring = [31, 32, 33, 255];
    host.text = [41, 42, 43, 255];
    host.text_muted = [51, 52, 53, 255];
    host.text_disabled = [61, 62, 63, 255];

    let palette = workbench_tree_row_palette_from_host(host);

    assert_eq!(palette.marked_surface, [9, 10, 11, 255]);
    assert_eq!(palette.inactive_marked_surface, [12, 13, 14, 255]);
    assert_eq!(palette.selected_text, [15, 16, 17, 255]);
    assert_eq!(palette.pressed_surface, [6, 7, 8, 255]);
    assert_eq!(palette.hot_surface, [21, 22, 23, 255]);
    assert_eq!(palette.focus_border, [31, 32, 33, 255]);
    assert_eq!(palette.text, [41, 42, 43, 255]);
    assert_eq!(palette.text_muted, [51, 52, 53, 255]);
    assert_eq!(palette.text_disabled, [61, 62, 63, 255]);
}

#[test]
fn selected_tree_row_uses_owner_focus_without_row_focus_or_hover_inference() {
    let mut node = TemplatePaneNodeData {
        selected: true,
        tree_owner_focused: Some(true),
        ..Default::default()
    };
    assert_eq!(
        select_workbench_tree_row_style(&node).background,
        Some(PALETTE.surface_selected)
    );
    node.tree_owner_focused = Some(false);
    node.focused = true;
    node.hovered = true;
    assert_eq!(
        select_workbench_tree_row_style(&node).background,
        Some(PALETTE.surface_selected_inactive)
    );
    assert_eq!(
        select_workbench_tree_row_style(&node).text,
        PALETTE.text_selected
    );
    node.disabled = true;
    assert_eq!(select_workbench_tree_row_style(&node).background, None);
}
