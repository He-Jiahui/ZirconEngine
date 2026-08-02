use super::super::workbench_row_selection::selected_row_outline_color;
use super::palette::{WorkbenchTreeRowPalette, workbench_tree_row_palette};
use super::state::{is_focus_or_press, is_hot, is_unavailable_tree_row_state};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    tree_row_background_from_palette(node, state, marked, workbench_tree_row_palette())
}

fn tree_row_background_from_palette(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchTreeRowPalette,
) -> Option<[u8; 4]> {
    if is_unavailable_tree_row_state(state) {
        None
    } else if state == UiPainterResolvedState::Pressed {
        Some(palette.pressed_surface)
    } else if marked {
        if node.hovered {
            Some(palette.marked_hot_surface)
        } else {
            Some(palette.marked_surface)
        }
    } else if is_hot(state) {
        Some(palette.hot_surface)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border(
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    tree_row_border_from_palette(state, marked, workbench_tree_row_palette())
}

fn tree_row_border_from_palette(
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchTreeRowPalette,
) -> Option<[u8; 4]> {
    if is_unavailable_tree_row_state(state) {
        None
    } else if marked {
        Some(selected_row_outline_color())
    } else if is_focus_or_press(state) {
        Some(palette.focus_border)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border_width(
    state: UiPainterResolvedState,
    marked: bool,
) -> f32 {
    if tree_row_border(state, marked).is_some() {
        1.0
    } else {
        0.0
    }
}
