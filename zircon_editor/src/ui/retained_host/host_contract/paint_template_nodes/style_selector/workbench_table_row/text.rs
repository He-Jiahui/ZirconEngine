//! 表格四列文字按行身份与列位置选择角色；第 4 列表尾值允许单独声明色，禁用/加载仍优先。
//! 此列约定与 template_table_rows/cells/metrics.rs 的 TABLE_COLUMN_COUNT=4 及 cells/commands.rs 的索引消费绑定。

use super::model::WorkbenchTableRowStyle;
use super::palette::workbench_table_row_palette;
use super::state::is_unavailable_table_row_state;

impl WorkbenchTableRowStyle {
    /// 在表格绘制端按固定四列顺序调用；第 4 列表尾值有单独文字角色。
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
