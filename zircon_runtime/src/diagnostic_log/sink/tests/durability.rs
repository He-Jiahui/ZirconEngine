use std::time::Duration;

use super::super::worker::SinkRuntime;
use super::fixtures::{FailingOutput, SyncFailingOutput};
use crate::diagnostic_log::{DiagnosticLogLevel, DiagnosticLogSinkSettings};

#[test]
fn shutdown_reports_write_failure_without_counting_records_as_written() {
    let runtime = SinkRuntime::start(
        Some(Box::new(FailingOutput)),
        false,
        DiagnosticLogSinkSettings::default().with_max_batch_records(1),
    )
    .expect("sink worker");

    assert!(runtime.enqueue(DiagnosticLogLevel::Error, "runtime", "must persist"));
    assert!(!runtime.shutdown(Duration::from_secs(2)));

    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.dequeued_records, 1);
    assert_eq!(snapshot.written_records, 0);
    assert!(snapshot.output_errors >= 1);
    assert!(snapshot.closed);
}

#[test]
fn shutdown_reports_sync_only_failure() {
    let runtime = SinkRuntime::start(
        Some(Box::new(SyncFailingOutput::default())),
        false,
        DiagnosticLogSinkSettings::default(),
    )
    .expect("sink worker");
    assert!(runtime.enqueue(DiagnosticLogLevel::Error, "runtime", "sync boundary"));

    assert!(!runtime.shutdown(Duration::from_secs(2)));
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.written_records, 1);
    assert!(snapshot.output_errors >= 1);
}

#[test]
fn partial_console_success_does_not_count_failed_file_mirror_as_written() {
    let runtime = SinkRuntime::start(
        Some(Box::new(FailingOutput)),
        true,
        DiagnosticLogSinkSettings::default().with_max_batch_records(1),
    )
    .expect("sink worker");
    assert!(runtime.enqueue(DiagnosticLogLevel::Error, "runtime", "partial"));

    assert!(!runtime.shutdown(Duration::from_secs(2)));
    assert_eq!(runtime.snapshot().written_records, 0);
}
