use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::state::is_unavailable_table_row_state;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_border(
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    if is_unavailable_table_row_state(state) {
        return None;
    }
    if marked
        && !matches!(
            state,
            UiPainterResolvedState::Dragging | UiPainterResolvedState::DropHovered
        )
    {
        return None;
    }
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
    .then_some(PALETTE.focus_ring)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_border_width(
    state: UiPainterResolvedState,
    marked: bool,
) -> f32 {
    if table_row_border(state, marked).is_some() {
        1.0
    } else {
        0.0
    }
}
