use super::super::workbench_row_selection::selected_row_outline_color;
use super::state::{is_focus_or_press, is_hot, is_unavailable_tree_row_state};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_background(
    _node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    let palette = current_host_palette();
    if is_unavailable_tree_row_state(state) {
        None
    } else if marked {
        Some(palette.surface_pressed)
    } else if state == UiPainterResolvedState::Pressed {
        Some(palette.surface_pressed)
    } else if is_hot(state) {
        Some(palette.surface_hover)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border(
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    let palette = current_host_palette();
    if is_unavailable_tree_row_state(state) {
        None
    } else if marked {
        Some(selected_row_outline_color())
    } else if is_focus_or_press(state) {
        Some(palette.focus_ring)
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
