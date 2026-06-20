use super::model::WorkbenchTableRowStyle;
use super::palette::WORKBENCH_TABLE_HEADER_TEXT;
use super::state::is_unavailable_table_row_state;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

impl WorkbenchTableRowStyle {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_for_cell(
        self,
        index: usize,
    ) -> [u8; 4] {
        if is_unavailable_table_row_state(self.state) {
            PALETTE.text_disabled
        } else if self.header {
            WORKBENCH_TABLE_HEADER_TEXT
        } else if self.tail && index == 3 {
            self.tail_value_text
        } else if index >= 2 {
            self.muted_text
        } else {
            self.text
        }
    }
}
