use super::super::workbench_row_selection::selected_row_outline_color;
use super::palette::{workbench_list_row_palette, WorkbenchListRowPalette};
use super::state::is_unavailable_list_row_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    list_row_background_from_palette(node, state, marked, workbench_list_row_palette())
}

fn list_row_background_from_palette(
    _node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchListRowPalette,
) -> Option<[u8; 4]> {
    if is_unavailable_list_row_state(state) {
        None
    } else if state == UiPainterResolvedState::Pressed {
        Some(palette.pressed_surface)
    } else if marked {
        Some(palette.marked_surface)
    } else {
        match state {
            UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
            | UiPainterResolvedState::Hovered => Some(palette.hot_surface),
            UiPainterResolvedState::Focused
            | UiPainterResolvedState::Disabled
            | UiPainterResolvedState::Loading
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => None,
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_border(
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    list_row_border_from_palette(state, marked, workbench_list_row_palette())
}

fn list_row_border_from_palette(
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchListRowPalette,
) -> Option<[u8; 4]> {
    if !is_unavailable_list_row_state(state) && marked {
        return Some(selected_row_outline_color());
    }
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
    .then_some(palette.focus_border)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_border_width(
    state: UiPainterResolvedState,
    marked: bool,
) -> f32 {
    if list_row_border(state, marked).is_some() {
        1.0
    } else {
        0.0
    }
}
