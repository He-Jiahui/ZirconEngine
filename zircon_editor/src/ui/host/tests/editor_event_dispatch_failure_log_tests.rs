use super::emit_failed_event_log;
use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord, EditorEventResult,
    EditorEventSequence, EditorEventSource, EditorEventUndoPolicy, MenuAction,
};
use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

fn failed_record(error: impl Into<String>) -> EditorEventRecord {
    EditorEventRecord {
        event_id: EditorEventId::new(7),
        sequence: EditorEventSequence::new(11),
        source: EditorEventSource::RetainedHost,
        event: EditorEvent::WorkbenchMenu(MenuAction::SaveProject),
        binding_path: Some("WorkbenchMenu/SaveProject".to_string()),
        operation_id: Some("project.save".to_string()),
        operation_display_name: Some("Save Project".to_string()),
        operation_arguments: None,
        operation_group: None,
        transaction_id: None,
        save_generation: None,
        effects: vec![EditorEventEffect::PresentationChanged],
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: 4,
        after_revision: 4,
        result: EditorEventResult::failure(error),
    }
}

#[test]
fn failed_editor_event_emits_a_structured_error_log() {
    let logs = EditorLogService::default();

    emit_failed_event_log(&logs, &failed_record("disk is read-only"));

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    let entry = records[0].entry();
    assert_eq!(entry.source(), &LogSource::editor());
    assert_eq!(entry.severity(), LogSeverity::Error);
    assert_eq!(entry.timestamp_frame(), 0);
    assert_eq!(
        entry.message(),
        "Editor event `project.save` failed: disk is read-only"
    );
}

#[test]
fn oversized_editor_event_diagnostic_uses_a_bounded_fallback_log() {
    let logs = EditorLogService::default();
    let oversized_error = "x".repeat(9 * 1024);

    emit_failed_event_log(&logs, &failed_record(oversized_error));

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].entry().message(),
        "Editor event 11 failed; diagnostic exceeds the log-entry limit."
    );
}

#[test]
fn authoring_trace_uses_the_editor_log_service() {
    let logs = EditorLogService::default();
    let record = failed_record("disk is read-only");

    super::emit_mvp_authoring_product_trace(&logs, &record, "failed");

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    let entry = records[0].entry();
    assert_eq!(entry.source(), &LogSource::editor());
    assert_eq!(entry.severity(), LogSeverity::Info);
    assert_eq!(entry.timestamp_frame(), 0);
    assert_eq!(
        entry.message(),
        "editor_authoring_trace result=failed event=save_project binding=WorkbenchMenu/SaveProject operation=project.save transaction_id=none save_generation=none"
    );
}

#[test]
fn oversized_authoring_trace_uses_a_bounded_fallback_log() {
    let logs = EditorLogService::default();
    let mut record = failed_record("disk is read-only");
    record.binding_path = Some("x".repeat(9 * 1024));

    super::emit_mvp_authoring_product_trace(&logs, &record, "failed");

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    let entry = records[0].entry();
    assert_eq!(entry.source(), &LogSource::editor());
    assert_eq!(entry.severity(), LogSeverity::Info);
    assert_eq!(
        entry.message(),
        "editor_authoring_trace result=failed event=save_project sequence=11 diagnostic exceeds the log-entry limit."
    );
}
