use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn focused_tree_view_does_not_mark_second_row_hovered() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    assert_eq!(tree_view_row_color(&node, 0), PALETTE.surface);
    assert_eq!(tree_view_row_color(&node, 1), PALETTE.surface);
}

#[test]
fn expanded_tree_view_still_marks_second_row_hovered() {
    let mut node = TemplatePaneNodeData::default();
    node.expanded = true;

    assert_eq!(tree_view_row_color(&node, 1), PALETTE.surface_hover);
}

#[test]
fn popup_open_tree_view_still_marks_second_row_hovered() {
    let mut node = TemplatePaneNodeData::default();
    node.popup_open = true;

    assert_eq!(tree_view_row_color(&node, 1), PALETTE.surface_hover);
}

#[test]
fn selected_tree_view_still_marks_first_row_selected() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;

    assert_eq!(tree_view_row_color(&node, 0), PALETTE.surface_selected);
}

#[test]
fn mui_x_tree_view_surface_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.success_container = [10, 11, 12, 255];
    palette.surface_inset = [20, 21, 22, 255];

    let mut selected = TemplatePaneNodeData::default();
    selected.selected = true;

    assert_eq!(
        tree_view_surface_color_from_host(&selected, palette),
        [10, 11, 12, 255]
    );
    assert_eq!(
        tree_view_surface_color_from_host(&TemplatePaneNodeData::default(), palette),
        [20, 21, 22, 255]
    );
}

#[test]
fn mui_x_tree_view_row_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_selected = [10, 11, 12, 255];
    palette.surface_hover = [20, 21, 22, 255];
    palette.surface = [30, 31, 32, 255];

    let mut selected = TemplatePaneNodeData::default();
    selected.selected = true;
    let mut expanded = TemplatePaneNodeData::default();
    expanded.expanded = true;

    assert_eq!(
        tree_view_row_color_from_host(&selected, 0, palette),
        [10, 11, 12, 255]
    );
    assert_eq!(
        tree_view_row_color_from_host(&expanded, 1, palette),
        [20, 21, 22, 255]
    );
    assert_eq!(
        tree_view_row_color_from_host(&TemplatePaneNodeData::default(), 1, palette),
        [30, 31, 32, 255]
    );
}

#[test]
fn mui_x_tree_view_marker_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.success = [10, 11, 12, 255];
    palette.border = [20, 21, 22, 255];

    let mut expanded = TemplatePaneNodeData::default();
    expanded.expanded = true;

    assert_eq!(
        tree_view_marker_color_from_host(&expanded, 0, palette),
        [10, 11, 12, 255]
    );
    assert_eq!(
        tree_view_marker_color_from_host(&TemplatePaneNodeData::default(), 0, palette),
        [20, 21, 22, 255]
    );
}
