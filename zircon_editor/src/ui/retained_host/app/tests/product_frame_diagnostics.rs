use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

use super::{emit_product_frame_log, product_frame_project_path_token};

#[cfg(windows)]
#[test]
fn product_frame_diagnostic_uses_a_display_path_for_verbatim_project_roots() {
    let token = product_frame_project_path_token(r"\\?\C:\projects\renderable empty");

    assert_eq!(token, "C%3A%5Cprojects%5Crenderable%20empty");
}

#[test]
fn oversized_product_frame_diagnostic_uses_a_bounded_info_log() {
    let logs = EditorLogService::default();

    emit_product_frame_log(&logs, "x".repeat(9 * 1024));

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    let entry = records[0].entry();
    assert_eq!(entry.source(), &LogSource::editor());
    assert_eq!(entry.severity(), LogSeverity::Info);
    assert_eq!(
        entry.message(),
        "editor_product_frame diagnostic exceeds the log-entry limit."
    );
}
