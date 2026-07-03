use super::model::WorkbenchTableRowStyle;
use super::palette::workbench_table_row_palette;
use super::state::is_unavailable_table_row_state;

impl WorkbenchTableRowStyle {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_for_cell(
        self,
        index: usize,
    ) -> [u8; 4] {
        let palette = workbench_table_row_palette();
        if is_unavailable_table_row_state(self.state) {
            palette.text_disabled
        } else if self.header {
            palette.header_text
        } else if self.tail && index == 3 {
            self.tail_value_text
        } else if index >= 2 {
            self.muted_text
        } else {
            self.text
        }
    }
}
