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
