use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use zircon_runtime::core::{
    TaskDiagnosticSeverity, TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES, TASK_DIAGNOSTIC_RETENTION_CAPACITY,
};

use super::{bridge::log_severity, RuntimeTaskDiagnosticLogBridge};
use crate::core::jobs::test_job_scheduler;
use crate::core::logging::{EditorLogService, LogChannel, LogFilter, LogSeverity};

#[test]
fn panic_observation_is_projected_once_through_the_canonical_runtime_log_source() {
    let scheduler = test_job_scheduler();
    let source = scheduler.task_diagnostic_source();
    let mut bridge = RuntimeTaskDiagnosticLogBridge::new(source);
    let logs = Arc::new(EditorLogService::default());

    let handle = scheduler.schedule(|| panic!("runtime task bridge panic"));
    let wait_result = catch_unwind(AssertUnwindSafe(|| handle.wait()));
    assert!(wait_result.is_err());

    let first = bridge
        .pump(logs.as_ref(), 17)
        .expect("the editor log service should accept the runtime diagnostic");
    let repeated = bridge
        .pump(logs.as_ref(), 18)
        .expect("an unchanged cursor should remain a successful no-op");

    assert_eq!(first.observation_count(), 1);
    assert_eq!(repeated.observation_count(), 0);
    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().source().channel(), LogChannel::Runtime);
    assert_eq!(records[0].entry().severity(), LogSeverity::Error);
    assert_eq!(records[0].entry().timestamp_frame(), 17);
    assert!(records[0]
        .entry()
        .message()
        .contains("runtime task bridge panic"));
}

#[test]
fn cancellation_severity_projects_to_editor_warning() {
    assert_eq!(
        log_severity(TaskDiagnosticSeverity::Warning),
        LogSeverity::Warning
    );
}

#[test]
fn lagged_cursor_emits_one_gap_warning_before_the_bounded_observation_batch() {
    let scheduler = test_job_scheduler();
    let source = scheduler.task_diagnostic_source();
    let mut bridge = RuntimeTaskDiagnosticLogBridge::new(source);
    let logs = Arc::new(EditorLogService::default());
    let handles = (0..TASK_DIAGNOSTIC_RETENTION_CAPACITY + 3)
        .map(|index| scheduler.schedule(move || panic!("retained panic {index}")))
        .collect::<Vec<_>>();
    for handle in handles {
        let wait_result = catch_unwind(AssertUnwindSafe(|| handle.wait()));
        assert!(wait_result.is_err());
    }

    let report = bridge
        .pump(logs.as_ref(), 23)
        .expect("the editor log service should accept the gap and retained observations");

    assert_eq!(report.gap_record_count(), 1);
    assert_eq!(report.dropped_observation_count(), 3);
    assert_eq!(
        report.observation_count(),
        TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES
    );
    assert!(report.has_more());
    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES + 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Warning);
    assert!(records[0]
        .entry()
        .message()
        .contains("dropped 3 observations"));
}
