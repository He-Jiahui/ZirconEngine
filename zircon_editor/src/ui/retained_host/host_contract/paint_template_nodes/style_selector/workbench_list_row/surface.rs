use super::super::workbench_row_selection::selected_row_outline_color;
use super::state::is_unavailable_list_row_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_background(
    _node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    let palette = current_host_palette();
    if is_unavailable_list_row_state(state) {
        None
    } else if marked {
        Some(palette.surface_pressed)
    } else {
        match state {
            UiPainterResolvedState::Pressed => Some(palette.surface_pressed),
            UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
            | UiPainterResolvedState::Hovered => Some(palette.surface_hover),
            UiPainterResolvedState::Disabled
            | UiPainterResolvedState::Loading
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
    let palette = current_host_palette();
    if !is_unavailable_list_row_state(state) && marked {
        return Some(selected_row_outline_color());
    }
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
    .then_some(palette.focus_ring)
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
