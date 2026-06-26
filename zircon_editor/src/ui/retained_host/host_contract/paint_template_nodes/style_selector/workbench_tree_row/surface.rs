use super::state::{is_focus_or_press, is_hot, is_unavailable_tree_row_state};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_background(
    _node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    if is_unavailable_tree_row_state(state) {
        None
    } else if marked {
        Some(PALETTE.surface_pressed)
    } else if state == UiPainterResolvedState::Pressed {
        Some(PALETTE.surface_pressed)
    } else if is_hot(state) {
        Some(PALETTE.surface_hover)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border(
    state: UiPainterResolvedState,
    _marked: bool,
) -> Option<[u8; 4]> {
    (!is_unavailable_tree_row_state(state) && is_focus_or_press(state))
        .then_some(PALETTE.focus_ring)
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
