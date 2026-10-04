/// 一次布局会话对富文本表格各阶段的工作量回执，供诊断与性能画像读取。
/// 计数是观测值而非准入预算；溢出饱和避免遥测改变布局控制流。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextTableLayoutWorkReport {
    pub(crate) table_layout_attempt_count: usize,
    pub(crate) table_source_byte_count: usize,
    pub(crate) table_cell_count: usize,
    pub(crate) max_table_cell_count: usize,
    pub(crate) preferred_cell_layout_count: usize,
    pub(crate) preferred_cell_input_bytes: usize,
    pub(crate) final_cell_layout_count: usize,
    pub(crate) final_cell_input_bytes: usize,
    pub(crate) column_track_count: usize,
    pub(crate) row_track_count: usize,
    pub(crate) published_line_count: usize,
    pub(crate) published_box_count: usize,
}

impl TextTableLayoutWorkReport {
    pub(crate) fn record_layout_attempt(&mut self, source_bytes: usize, cell_count: usize) {
        self.table_layout_attempt_count = self.table_layout_attempt_count.saturating_add(1);
        self.table_source_byte_count = self.table_source_byte_count.saturating_add(source_bytes);
        self.table_cell_count = self.table_cell_count.saturating_add(cell_count);
        self.max_table_cell_count = self.max_table_cell_count.max(cell_count);
    }

    pub(crate) fn record_tracks(&mut self, column_count: usize, row_count: usize) {
        self.column_track_count = self.column_track_count.saturating_add(column_count);
        self.row_track_count = self.row_track_count.saturating_add(row_count);
    }

    pub(crate) fn record_preferred_cell_layout(&mut self, source_bytes: usize) {
        self.preferred_cell_layout_count = self.preferred_cell_layout_count.saturating_add(1);
        self.preferred_cell_input_bytes =
            self.preferred_cell_input_bytes.saturating_add(source_bytes);
    }

    pub(crate) fn record_final_cell_layout(&mut self, source_bytes: usize) {
        self.final_cell_layout_count = self.final_cell_layout_count.saturating_add(1);
        self.final_cell_input_bytes = self.final_cell_input_bytes.saturating_add(source_bytes);
    }

    pub(crate) fn record_output(&mut self, line_count: usize, box_count: usize) {
        self.published_line_count = self.published_line_count.saturating_add(line_count);
        self.published_box_count = self.published_box_count.saturating_add(box_count);
    }

    pub(crate) fn publish_profile_counters(self) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            crate::profile_counter!(
                "runtime",
                "rich_table_layout_attempt_count",
                self.table_layout_attempt_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_source_byte_count",
                self.table_source_byte_count
            );
            crate::profile_counter!("runtime", "rich_table_cell_count", self.table_cell_count);
            crate::profile_counter!(
                "runtime",
                "rich_table_max_cell_count",
                self.max_table_cell_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_preferred_cell_layout_count",
                self.preferred_cell_layout_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_preferred_cell_input_bytes",
                self.preferred_cell_input_bytes
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_final_cell_layout_count",
                self.final_cell_layout_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_final_cell_input_bytes",
                self.final_cell_input_bytes
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_column_track_count",
                self.column_track_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_row_track_count",
                self.row_track_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_published_line_count",
                self.published_line_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_published_box_count",
                self.published_box_count
            );
        }
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = self;
    }
}

#[cfg(test)]
#[path = "tests/table_work.rs"]
mod tests;
