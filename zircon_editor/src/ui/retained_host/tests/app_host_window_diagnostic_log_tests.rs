use super::{emit_host_window_diagnostics, HostWindowDiagnostic, HostWindowDiagnosticSeverity};
use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

#[test]
fn native_window_diagnostics_keep_their_severity_at_the_editor_log_boundary() {
    let logs = EditorLogService::default();

    emit_host_window_diagnostics(
        &logs,
        vec![
            HostWindowDiagnostic::new(HostWindowDiagnosticSeverity::Info, "frame ready"),
            HostWindowDiagnostic::new(HostWindowDiagnosticSeverity::Error, "present failed"),
        ],
    );

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 2);
    assert!(records
        .iter()
        .all(|record| record.entry().source() == &LogSource::editor()));
    assert_eq!(records[0].entry().severity(), LogSeverity::Info);
    assert_eq!(records[0].entry().message(), "frame ready");
    assert_eq!(records[1].entry().severity(), LogSeverity::Error);
    assert_eq!(records[1].entry().message(), "present failed");
}

#[test]
fn oversized_native_window_diagnostic_uses_a_bounded_severity_preserving_fallback() {
    let logs = EditorLogService::default();

    emit_host_window_diagnostics(
        &logs,
        vec![HostWindowDiagnostic::new(
            HostWindowDiagnosticSeverity::Warning,
            "x".repeat(9 * 1024),
        )],
    );

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Warning);
    assert_eq!(
        records[0].entry().message(),
        "editor_host_window diagnostic exceeds the log-entry limit."
    );
}
