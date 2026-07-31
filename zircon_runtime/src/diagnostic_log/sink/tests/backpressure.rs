use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::super::worker::SinkRuntime;
use super::fixtures::{BlockingOutput, SharedOutput};
use crate::diagnostic_log::{DiagnosticLogLevel, DiagnosticLogSinkSettings};

#[test]
fn full_queue_drops_best_effort_logs_and_counts_them() {
    let writer = BlockingOutput::default();
    let runtime = SinkRuntime::start(
        Some(Box::new(writer.clone())),
        false,
        one_record_blocking_settings(),
    )
    .expect("sink worker");

    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "in-flight"));
    writer.wait_until_blocked();
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "queued"));
    assert!(!runtime.enqueue(DiagnosticLogLevel::Debug, "runtime", "dropped"));
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.queue_depth, 1);
    assert_eq!(snapshot.max_queue_depth, 1);
    assert_eq!(snapshot.dropped_debug, 1);

    writer.release();
    assert!(runtime.shutdown(Duration::from_secs(2)));
}

#[test]
fn full_queue_records_capacity_as_the_high_water_mark() {
    let writer = BlockingOutput::default();
    let runtime = SinkRuntime::start(
        Some(Box::new(writer.clone())),
        false,
        DiagnosticLogSinkSettings::default()
            .with_queue_capacity(2)
            .with_max_batch_records(1)
            .with_flush_interval(Duration::from_secs(60)),
    )
    .expect("sink worker");

    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "in-flight"));
    writer.wait_until_blocked();
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "queued-1"));
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "queued-2"));
    assert!(!runtime.enqueue(DiagnosticLogLevel::Debug, "runtime", "dropped"));

    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.queue_depth, 2);
    assert_eq!(snapshot.max_queue_depth, 2);

    writer.release();
    assert!(runtime.shutdown(Duration::from_secs(2)));
}

#[test]
fn fast_dequeue_without_a_full_send_still_records_queue_use() {
    let runtime = SinkRuntime::start(
        Some(Box::new(SharedOutput::default())),
        false,
        DiagnosticLogSinkSettings::default()
            .with_queue_capacity(8)
            .with_max_batch_records(1),
    )
    .expect("sink worker");

    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "single"));
    assert!(runtime.shutdown(Duration::from_secs(2)));
    assert!(runtime.snapshot().max_queue_depth >= 1);
}

#[test]
fn full_queue_applies_backpressure_to_warn_without_dropping_it() {
    let writer = BlockingOutput::default();
    let runtime = Arc::new(
        SinkRuntime::start(
            Some(Box::new(writer.clone())),
            false,
            one_record_blocking_settings(),
        )
        .expect("sink worker"),
    );

    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "in-flight"));
    writer.wait_until_blocked();
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "queued"));

    let (completed_tx, completed_rx) = mpsc::sync_channel(1);
    let producer_runtime = Arc::clone(&runtime);
    let producer = std::thread::spawn(move || {
        let accepted = producer_runtime.enqueue(DiagnosticLogLevel::Warn, "runtime", "durable");
        completed_tx.send(accepted).unwrap();
    });
    assert!(matches!(
        completed_rx.recv_timeout(Duration::from_millis(50)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));

    writer.release();
    assert!(completed_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    producer.join().unwrap();
    assert!(runtime.shutdown(Duration::from_secs(2)));

    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.dropped_warn, 0);
    assert!(snapshot.critical_backpressure_count >= 1);
    assert!(writer.text().contains("durable"));
}

fn one_record_blocking_settings() -> DiagnosticLogSinkSettings {
    DiagnosticLogSinkSettings::default()
        .with_queue_capacity(1)
        .with_max_batch_records(1)
        .with_flush_interval(Duration::from_secs(60))
}
