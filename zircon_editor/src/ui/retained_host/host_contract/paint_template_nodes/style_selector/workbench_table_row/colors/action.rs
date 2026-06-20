use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::palette::WORKBENCH_TABLE_ACTION_MUTED;
use super::super::state::is_unavailable_table_row_state;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_action_color(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_table_row_state(state) {
        PALETTE.text_disabled
    } else {
        WORKBENCH_TABLE_ACTION_MUTED
    }
}
