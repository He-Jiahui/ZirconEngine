use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::super::workbench_row_selection::selected_row_outline_color;
use super::super::palette::workbench_table_row_palette;
use super::super::state::is_unavailable_table_row_state;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_border(
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    let palette = workbench_table_row_palette();
    if is_unavailable_table_row_state(state) {
        return None;
    }
    if marked {
        return if matches!(
            state,
            UiPainterResolvedState::Dragging | UiPainterResolvedState::DropHovered
        ) {
            Some(palette.focus_ring)
        } else {
            Some(selected_row_outline_color())
        };
    }
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
    .then_some(palette.focus_ring)
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
