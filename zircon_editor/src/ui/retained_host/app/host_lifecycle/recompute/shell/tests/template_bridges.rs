use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

use super::emit_template_bridge_layout_error;

#[test]
fn template_bridge_recompute_failures_are_not_silently_discarded() {
    let source = include_str!("../template_bridges.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("test module should remain isolated from production bridge recompute code");

    assert!(
        !production.contains("let _ = self.template_bridge.recompute_layout_with_workbench_model")
    );
    assert!(!production.contains("let _ = self\n                .workbench_window_bridge"));
    assert!(production.contains("editor_root_template_bridge_layout"));
    assert!(production.contains("editor_workbench_template_bridge_layout"));
    assert!(
        production
            .matches("emit_template_bridge_layout_error(")
            .count()
            >= 2
    );
    assert!(!production.contains("diagnostic_log::write_error"));
    assert!(production.contains("ResolutionContext::from_physical_size_with_scale_mode"));
    assert!(production.contains("effective_scale_factor()"));
    assert!(production.contains("self.shell_scale_mode"));
    assert!(production.contains("recompute_layout_with_workbench_model_at_scale"));
}

#[test]
fn template_bridge_layout_failures_are_emitted_as_bounded_editor_errors() {
    let logs = EditorLogService::default();

    emit_template_bridge_layout_error(
        &logs,
        "editor_root_template_bridge_layout",
        "layout cache invalid",
    );
    emit_template_bridge_layout_error(
        &logs,
        "editor_workbench_template_bridge_layout",
        "x".repeat(9 * 1024),
    );

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 2);
    assert!(records
        .iter()
        .all(|record| record.entry().source() == &LogSource::editor()));
    assert!(records
        .iter()
        .all(|record| record.entry().severity() == LogSeverity::Error));
    assert_eq!(
        records[0].entry().message(),
        "editor_root_template_bridge_layout layout cache invalid"
    );
    assert_eq!(
        records[1].entry().message(),
        "editor_template_bridge layout diagnostic exceeds the log-entry limit."
    );
}
