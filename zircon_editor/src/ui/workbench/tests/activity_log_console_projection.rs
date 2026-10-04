use crate::core::logging::{EditorLogConfig, LogEntry, LogSource};

use super::*;

#[test]
fn append_reuses_retained_chunks_and_publishes_an_exact_sequence_delta() {
    let logs = EditorLogService::new(
        EditorLogConfig::new(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY + 1, 128 * 1024).unwrap(),
    );
    for index in 0..CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY {
        emit(&logs, index);
    }
    let mut projection = ActivityLogConsoleProjection::default();
    let before = projection.project(&logs, ConsoleMessageFilter::All, ConsoleSourceFilter::All);
    emit(&logs, CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);

    let after = projection.project(&logs, ConsoleMessageFilter::All, ConsoleSourceFilter::All);

    assert_eq!(
        after.line_delta(),
        ConsoleOutputLineDelta {
            entered: 1,
            expired: 1,
            retained: CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY - 1,
        }
    );
    assert!(before.shares_logical_storage_chunk_with(&after, 64, 63));
    assert!(!after.has_materialized_flat_text());

    let unchanged = projection.project(&logs, ConsoleMessageFilter::All, ConsoleSourceFilter::All);
    assert!(after.shares_logical_generation_with(&unchanged));
    assert_eq!(unchanged.line_delta(), after.line_delta());
}

#[test]
fn activity_filter_uses_the_direct_channel_mask_constructor() {
    let source = include_str!("../activity_log_console_projection.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    assert!(production.contains("LogFilter::from_channel("));
    assert!(!production.contains("BTreeSet"));
}

fn emit(logs: &EditorLogService, index: usize) {
    logs.emit(
        LogEntry::new(
            LogSource::editor(),
            LogSeverity::Info,
            format!("record-{index}"),
            index as u64,
            None,
        )
        .unwrap(),
    )
    .unwrap();
}
