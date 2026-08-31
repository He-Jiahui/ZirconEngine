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
mod tests {
    use super::TextTableLayoutWorkReport;

    #[test]
    fn report_counts_actual_table_layout_phases_without_implying_a_budget() {
        let mut report = TextTableLayoutWorkReport::default();

        report.record_layout_attempt(120, 3);
        report.record_tracks(2, 2);
        report.record_preferred_cell_layout(10);
        report.record_preferred_cell_layout(20);
        report.record_final_cell_layout(10);
        report.record_output(4, 3);

        assert_eq!(report.table_layout_attempt_count, 1);
        assert_eq!(report.table_source_byte_count, 120);
        assert_eq!(report.table_cell_count, 3);
        assert_eq!(report.max_table_cell_count, 3);
        assert_eq!(report.preferred_cell_layout_count, 2);
        assert_eq!(report.preferred_cell_input_bytes, 30);
        assert_eq!(report.final_cell_layout_count, 1);
        assert_eq!(report.final_cell_input_bytes, 10);
        assert_eq!(report.column_track_count, 2);
        assert_eq!(report.row_track_count, 2);
        assert_eq!(report.published_line_count, 4);
        assert_eq!(report.published_box_count, 3);
    }

    #[test]
    fn report_saturates_telemetry_counters() {
        let mut report = TextTableLayoutWorkReport {
            table_layout_attempt_count: usize::MAX,
            table_source_byte_count: usize::MAX,
            table_cell_count: usize::MAX,
            preferred_cell_layout_count: usize::MAX,
            preferred_cell_input_bytes: usize::MAX,
            final_cell_layout_count: usize::MAX,
            final_cell_input_bytes: usize::MAX,
            column_track_count: usize::MAX,
            row_track_count: usize::MAX,
            published_line_count: usize::MAX,
            published_box_count: usize::MAX,
            ..Default::default()
        };

        report.record_layout_attempt(1, 1);
        report.record_tracks(1, 1);
        report.record_preferred_cell_layout(1);
        report.record_final_cell_layout(1);
        report.record_output(1, 1);

        assert_eq!(report.table_layout_attempt_count, usize::MAX);
        assert_eq!(report.table_source_byte_count, usize::MAX);
        assert_eq!(report.table_cell_count, usize::MAX);
        assert_eq!(report.preferred_cell_layout_count, usize::MAX);
        assert_eq!(report.final_cell_layout_count, usize::MAX);
        assert_eq!(report.column_track_count, usize::MAX);
        assert_eq!(report.row_track_count, usize::MAX);
        assert_eq!(report.published_line_count, usize::MAX);
        assert_eq!(report.published_box_count, usize::MAX);
    }
}
