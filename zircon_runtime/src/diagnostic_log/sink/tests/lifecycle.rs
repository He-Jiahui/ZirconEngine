use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use super::super::worker::SinkRuntime;
use super::super::DiagnosticLogState;
use super::fixtures::{BlockingOutput, SharedOutput};
use crate::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel, DiagnosticLogSinkSettings,
};

#[test]
fn disabled_lazy_log_does_not_evaluate_message() {
    let state =
        DiagnosticLogState::for_test(DiagnosticLogFilterConfig::new(DiagnosticLogFilter::Off));
    let evaluated = AtomicBool::new(false);
    state.write_lazy(DiagnosticLogLevel::Debug, "runtime::disabled", || {
        evaluated.store(true, Ordering::Relaxed);
        "must not run"
    });
    assert!(!evaluated.load(Ordering::Relaxed));
}

#[test]
fn lazy_log_does_not_evaluate_without_outputs_or_after_shutdown() {
    let state = DiagnosticLogState::for_test(DiagnosticLogFilterConfig::new(
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose),
    ));
    let evaluated_without_outputs = AtomicBool::new(false);
    state.write_lazy(DiagnosticLogLevel::Log, "runtime", || {
        evaluated_without_outputs.store(true, Ordering::Relaxed);
        "unused"
    });
    assert!(!evaluated_without_outputs.load(Ordering::Relaxed));

    let runtime = SinkRuntime::start(
        Some(Box::new(SharedOutput::default())),
        false,
        DiagnosticLogSinkSettings::default(),
    )
    .expect("sink worker");
    assert!(runtime.shutdown(Duration::from_secs(2)));
    let evaluated_after_shutdown = AtomicBool::new(false);
    assert!(
        !runtime.enqueue_lazy(DiagnosticLogLevel::Log, "runtime", || {
            evaluated_after_shutdown.store(true, Ordering::Relaxed);
            "unused"
        })
    );
    assert!(!evaluated_after_shutdown.load(Ordering::Relaxed));
}

#[test]
fn lazy_message_panic_releases_sender_before_shutdown() {
    let runtime = SinkRuntime::start(
        Some(Box::new(SharedOutput::default())),
        false,
        DiagnosticLogSinkSettings::default(),
    )
    .expect("sink worker");

    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.enqueue_lazy(DiagnosticLogLevel::Log, "runtime", || -> &'static str {
            panic!("message construction failed")
        });
    }));

    assert!(panic.is_err());
    assert!(runtime.shutdown(Duration::from_secs(2)));
}

#[test]
fn shutdown_flushes_a_pending_batch_in_fifo_order() {
    let output = SharedOutput::default();
    let runtime = SinkRuntime::start(
        Some(Box::new(output.clone())),
        false,
        DiagnosticLogSinkSettings::default()
            .with_queue_capacity(8)
            .with_max_batch_records(8)
            .with_flush_interval(Duration::from_secs(60)),
    )
    .expect("sink worker");

    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "first"));
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "second"));
    assert!(runtime.shutdown(Duration::from_secs(2)));

    let output = output.text();
    assert!(output.find("first").unwrap() < output.find("second").unwrap());
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.written_records, 2);
    assert_eq!(snapshot.flush_batches, 1);
    assert_eq!(snapshot.queue_depth, 0);
}

#[test]
fn shutdown_timeout_can_retry_after_the_worker_finishes() {
    let output = BlockingOutput::default();
    let runtime = SinkRuntime::start(
        Some(Box::new(output.clone())),
        false,
        DiagnosticLogSinkSettings::default().with_max_batch_records(1),
    )
    .expect("sink worker");
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "blocked"));
    output.wait_until_blocked();

    assert!(!runtime.shutdown(Duration::from_millis(20)));
    output.release();
    assert!(runtime.shutdown(Duration::from_secs(2)));
}

#[test]
fn concurrent_shutdown_callers_both_observe_completion() {
    let runtime = Arc::new(
        SinkRuntime::start(
            Some(Box::new(SharedOutput::default())),
            false,
            DiagnosticLogSinkSettings::default(),
        )
        .expect("sink worker"),
    );
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "final"));

    let start = Arc::new(Barrier::new(3));
    let first_runtime = Arc::clone(&runtime);
    let first_start = Arc::clone(&start);
    let first = std::thread::spawn(move || {
        first_start.wait();
        first_runtime.shutdown(Duration::from_secs(2))
    });
    let second_runtime = Arc::clone(&runtime);
    let second_start = Arc::clone(&start);
    let second = std::thread::spawn(move || {
        second_start.wait();
        second_runtime.shutdown(Duration::from_secs(2))
    });
    start.wait();

    assert!(first.join().unwrap());
    assert!(second.join().unwrap());
}
