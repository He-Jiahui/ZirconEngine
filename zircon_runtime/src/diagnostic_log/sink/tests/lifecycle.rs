use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use super::super::worker::SinkRuntime;
use super::super::{DiagnosticLogState, ProcessLogController};
use super::fixtures::{BlockingOutput, FailingOutput, SharedOutput};
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
fn library_shutdown_joins_the_worker_when_durable_output_reports_an_error() {
    let runtime = SinkRuntime::start(
        Some(Box::new(FailingOutput)),
        false,
        DiagnosticLogSinkSettings::default().with_max_batch_records(1),
    )
    .expect("sink worker");
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "final"));

    assert!(runtime.shutdown_for_library_unload(Duration::from_secs(2)));
    assert!(runtime.snapshot().closed);
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

#[test]
fn process_shutdown_without_an_active_sink_is_already_complete() {
    let controller = ProcessLogController::default();

    assert!(controller.shutdown_when_idle(Duration::from_millis(1)));
}

#[test]
fn runtime44_batch_borrowed_active_state_reads_do_not_clone_the_published_arc() {
    let controller = ProcessLogController::default();
    let state = Arc::new(DiagnosticLogState::for_test(
        DiagnosticLogFilterConfig::default(),
    ));
    controller.active_state.store(Some(Arc::clone(&state)));
    let strong_count = Arc::strong_count(&state);

    controller.with_active_state(|active| {
        assert!(active.is_some_and(|active| std::ptr::eq(active, state.as_ref())));
        assert_eq!(Arc::strong_count(&state), strong_count);
    });

    assert_eq!(Arc::strong_count(&state), strong_count);
}

#[test]
#[ignore = "managed release performance evidence"]
fn runtime44_batch_borrowed_active_state_evidence() {
    const READS: usize = 1_000_000;
    const MAX_ELAPSED_NS: u128 = 3_000_000_000;

    let controller = ProcessLogController::default();
    let state = Arc::new(DiagnosticLogState::for_test(
        DiagnosticLogFilterConfig::default(),
    ));
    controller.active_state.store(Some(Arc::clone(&state)));
    let strong_count = Arc::strong_count(&state);
    let started = Instant::now();
    for _ in 0..READS {
        black_box(controller.with_active_state(|active| active.is_some()));
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let legacy_arc_refcount_pairs = READS;
    let borrowed_arc_refcount_pairs = 0;
    let refcount_reduction_bps = 10_000;

    println!(
        "RUNTIME44_BORROWED_ACTIVE_STATE_BENCH_V1 reads={READS} legacy_arc_refcount_pairs={legacy_arc_refcount_pairs} borrowed_arc_refcount_pairs={borrowed_arc_refcount_pairs} refcount_reduction_bps={refcount_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );

    assert_eq!(Arc::strong_count(&state), strong_count);
    assert_eq!(borrowed_arc_refcount_pairs, 0);
    assert_eq!(refcount_reduction_bps, 10_000);
    assert!(elapsed_ns <= MAX_ELAPSED_NS);
}

#[test]
fn dynamic_session_leases_join_the_final_worker_and_allow_a_fresh_generation() {
    let controller = ProcessLogController::default();
    let first_output = SharedOutput::default();
    let first = controller.acquire_dynamic_session_for_test(|| {
        DiagnosticLogState::with_test_sink(
            SinkRuntime::start(
                Some(Box::new(first_output.clone())),
                false,
                DiagnosticLogSinkSettings::default(),
            )
            .expect("first sink worker"),
        )
    });
    let second = controller.acquire_dynamic_session_for_test(|| {
        panic!("a second dynamic session must reuse the live sink generation")
    });

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(controller.dynamic_session_count_for_test(), 2);
    assert!(controller.release_dynamic_session_for_test());
    assert_eq!(controller.dynamic_session_count_for_test(), 1);
    assert!(first.sink.as_ref().is_some_and(SinkRuntime::is_open));

    assert!(controller.release_dynamic_session_for_test());
    assert_eq!(controller.dynamic_session_count_for_test(), 0);
    assert!(first
        .sink
        .as_ref()
        .is_some_and(|sink| sink.snapshot().closed));
    assert!(controller.active_state_for_test().is_none());

    let next = controller.acquire_dynamic_session_for_test(|| {
        DiagnosticLogState::with_test_sink(
            SinkRuntime::start(
                Some(Box::new(SharedOutput::default())),
                false,
                DiagnosticLogSinkSettings::default(),
            )
            .expect("next sink worker"),
        )
    });
    assert!(!Arc::ptr_eq(&first, &next));
    assert!(controller.release_dynamic_session_for_test());
}

#[test]
fn final_dynamic_session_release_retains_retry_authority_after_shutdown_timeout() {
    let controller = ProcessLogController::default();
    let output = BlockingOutput::default();
    let state = controller.acquire_dynamic_session_for_test(|| {
        DiagnosticLogState::with_test_sink(
            SinkRuntime::start(
                Some(Box::new(output.clone())),
                false,
                DiagnosticLogSinkSettings::default().with_max_batch_records(1),
            )
            .expect("sink worker"),
        )
    });
    assert!(state.sink.as_ref().is_some_and(|sink| sink.enqueue(
        DiagnosticLogLevel::Log,
        "runtime",
        "blocked"
    )));
    output.wait_until_blocked();

    assert!(!controller.release_dynamic_session_with_timeout_for_test(Duration::from_millis(20)));
    assert_eq!(controller.dynamic_session_count_for_test(), 1);
    assert!(controller
        .active_state_for_test()
        .is_some_and(|active| Arc::ptr_eq(&active, &state)));

    output.release();
    assert!(controller.release_dynamic_session_for_test());
    assert_eq!(controller.dynamic_session_count_for_test(), 0);
    assert!(controller.active_state_for_test().is_none());
}
