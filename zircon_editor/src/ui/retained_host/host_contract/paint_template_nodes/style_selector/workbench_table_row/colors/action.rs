use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::palette::workbench_table_row_palette;
use super::super::state::is_unavailable_table_row_state;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_action_color(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_table_row_palette();
    if is_unavailable_table_row_state(state) {
        palette.text_disabled
    } else {
        palette.action_muted
    }
}
