use super::state::is_unavailable_list_row_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_background(
    _node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    if is_unavailable_list_row_state(state) {
        None
    } else if marked {
        Some(PALETTE.surface_pressed)
    } else {
        match state {
            UiPainterResolvedState::Pressed => Some(PALETTE.surface_pressed),
            UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
            | UiPainterResolvedState::Hovered => Some(PALETTE.surface_hover),
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
) -> Option<[u8; 4]> {
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
    .then_some(PALETTE.focus_ring)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_border_width(
    state: UiPainterResolvedState,
) -> f32 {
    if list_row_border(state).is_some() {
        1.0
    } else {
        0.0
    }
}
