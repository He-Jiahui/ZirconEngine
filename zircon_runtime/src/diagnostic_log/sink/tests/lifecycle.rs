use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use super::super::worker::SinkRuntime;
use super::super::{DiagnosticLogState, ProcessLogController};
use super::fixtures::{BlockingOutput, FailingOutput, SharedOutput};
use crate::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel, DiagnosticLogSettings,
    DiagnosticLogSinkSettings,
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
fn process_shutdown_explicitly_disabled_outputs_completes_without_worker() {
    let controller = ProcessLogController::default();
    let state = controller.initialize(
        DiagnosticLogSettings::new("disabled-shutdown")
            .with_filter(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose))
            .with_console_enabled(false)
            .with_file_enabled(false),
    );
    let evaluated = AtomicBool::new(false);
    state.write_lazy(DiagnosticLogLevel::Log, "runtime", || {
        evaluated.store(true, Ordering::Relaxed);
        "unused"
    });

    let stopped = controller.shutdown_when_idle(Duration::ZERO);
    let completed_result = controller.lock_lifecycle().completed_output_result;

    assert!(state.sink.is_none());
    assert!(state.file_path.is_none());
    assert!(state.snapshot().is_none());
    assert!(!evaluated.load(Ordering::Relaxed));
    assert!(stopped);
    assert!(controller.active_state_for_test().is_none());
    assert_eq!(completed_result, Some(true));
    assert!(controller.shutdown_when_idle(Duration::ZERO));
    assert!(controller.shutdown_when_idle(Duration::from_millis(1)));
}

#[test]
fn process_shutdown_missing_requested_sink_remains_failed() {
    for (console_enabled, file_enabled) in [(true, false), (false, true), (true, true)] {
        let controller = ProcessLogController::default();
        let mut missing_sink = DiagnosticLogState::for_test(DiagnosticLogFilterConfig::new(
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose),
        ));
        // Reproduce the postcondition of requested outputs failing to create a worker.
        missing_sink.console_enabled = console_enabled;
        missing_sink.file_enabled = file_enabled;
        let state = controller.publish_state(missing_sink, &mut controller.lock_lifecycle());

        let stopped = controller.shutdown_when_idle(Duration::ZERO);
        let completed_result = controller.lock_lifecycle().completed_output_result;

        assert!(state.sink.is_none());
        assert!(state.shutdown_for_library_unload(Duration::ZERO));
        assert!(controller.active_state_for_test().is_none());
        assert!(
            !stopped,
            "missing requested outputs must fail: console={console_enabled} file={file_enabled}"
        );
        assert_eq!(completed_result, Some(false));
        assert!(!controller.shutdown_when_idle(Duration::ZERO));
        assert!(!controller.shutdown_when_idle(Duration::from_millis(1)));
    }
}

#[test]
fn runtime44_batch_borrowed_active_state_reads_do_not_clone_the_published_arc() {
    let controller = ProcessLogController::default();
    let state = Arc::new(DiagnosticLogState::for_test(
        DiagnosticLogFilterConfig::new(DiagnosticLogFilter::default_for_build_profile()),
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
        DiagnosticLogFilterConfig::new(DiagnosticLogFilter::default_for_build_profile()),
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

#[test]
fn astra_life_a4_log_release_deadline_includes_controller_lock() {
    let controller = ProcessLogController::default();
    controller.acquire_dynamic_session_for_test(|| {
        DiagnosticLogState::with_test_sink(
            SinkRuntime::start(
                Some(Box::new(SharedOutput::default())),
                false,
                DiagnosticLogSinkSettings::default(),
            )
            .unwrap(),
        )
    });
    let lifecycle = controller.lock_lifecycle();
    let started_at = Instant::now();
    assert!(!controller.release_dynamic_session_with_timeout_for_test(Duration::from_millis(25)));
    assert!(started_at.elapsed() < Duration::from_secs(1));
    assert_eq!(lifecycle.dynamic_session_count, 1);
    drop(lifecycle);
    assert!(controller.release_dynamic_session_for_test());
}

#[test]
fn astra_life_a4_idle_log_shutdown_deadline_includes_controller_lock() {
    let controller = Arc::new(ProcessLogController::default());
    assert!(!controller.shutdown_when_idle(Duration::MAX));
    let (entered, entered_receiver) = std::sync::mpsc::sync_channel(1);
    let (release, release_receiver) = std::sync::mpsc::sync_channel(1);
    let other = Arc::clone(&controller);
    let holder = std::thread::spawn(move || {
        let _lifecycle = other.lock_lifecycle();
        entered.send(()).unwrap();
        let _ = release_receiver.recv_timeout(Duration::from_secs(2));
    });
    entered_receiver
        .recv_timeout(Duration::from_secs(1))
        .unwrap();

    let started_at = Instant::now();
    let stopped = controller.shutdown_when_idle(Duration::from_millis(25));
    let elapsed = started_at.elapsed();
    let _ = release.send(());
    holder.join().unwrap();

    assert!(!stopped);
    assert!(elapsed < Duration::from_secs(1));
    assert!(controller.shutdown_when_idle(Duration::ZERO));
}

#[test]
fn astra_life_a4_idle_log_shutdown_is_bounded_during_dynamic_release() {
    let controller = Arc::new(ProcessLogController::default());
    let output = BlockingOutput::default();
    let state = controller.acquire_dynamic_session_for_test(|| {
        DiagnosticLogState::with_test_sink(
            SinkRuntime::start(
                Some(Box::new(output.clone())),
                false,
                DiagnosticLogSinkSettings::default()
                    .with_max_batch_records(8)
                    .with_flush_interval(Duration::from_secs(60)),
            )
            .unwrap(),
        )
    });
    assert!(state.sink.as_ref().unwrap().enqueue(
        DiagnosticLogLevel::Log,
        "runtime",
        "final record"
    ));
    let other = Arc::clone(&controller);
    let release = std::thread::spawn(move || {
        other.release_dynamic_session_with_timeout_for_test(Duration::from_secs(2))
    });
    // The pending batch is written by shutdown while the release owns the lifecycle lock.
    output.wait_until_blocked();
    let started_at = Instant::now();
    let stopped = controller.shutdown_when_idle(Duration::from_millis(25));
    let elapsed = started_at.elapsed();
    output.release();
    let released = release.join().unwrap();
    if !released {
        assert!(controller.release_dynamic_session_for_test());
    }

    assert!(!stopped);
    assert!(elapsed < Duration::from_secs(1));
    assert!(released);
    assert_eq!(controller.dynamic_session_count_for_test(), 0);
    assert!(controller.active_state_for_test().is_none());
    assert!(state.sink.as_ref().unwrap().snapshot().closed);
    assert!(output.text().contains("final record"));
    assert!(controller.shutdown_when_idle(Duration::ZERO));
}

#[test]
fn astra_life_a4_log_worker_retains_join_after_shutdown_ack_until_output_drop_finishes() {
    struct DropBlockedOutput {
        entered: std::sync::mpsc::SyncSender<()>,
        release: std::sync::mpsc::Receiver<()>,
    }

    impl std::io::Write for DropBlockedOutput {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl super::super::worker::DurableOutput for DropBlockedOutput {
        fn sync_data(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Drop for DropBlockedOutput {
        fn drop(&mut self) {
            let _ = self.entered.send(());
            let _ = self.release.recv();
        }
    }

    let (entered_tx, entered_rx) = std::sync::mpsc::sync_channel(1);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let runtime = SinkRuntime::start(
        Some(Box::new(DropBlockedOutput {
            entered: entered_tx,
            release: release_rx,
        })),
        false,
        DiagnosticLogSinkSettings::default(),
    )
    .unwrap();

    let started_at = Instant::now();
    assert!(!runtime.shutdown_for_library_unload(Duration::from_millis(25)));
    assert!(started_at.elapsed() < Duration::from_secs(1));
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(runtime.snapshot().closed);
    assert!(!runtime.shutdown_for_library_unload(Duration::ZERO));

    release_tx.send(()).unwrap();
    assert!(runtime.shutdown_for_library_unload(Duration::from_secs(1)));
    assert!(runtime.shutdown_for_library_unload(Duration::ZERO));
}

#[cfg(windows)]
#[test]
fn astra_life_a4_log_worker_retains_join_while_tls_destruction_is_blocked() {
    use std::cell::RefCell;
    use std::sync::mpsc;

    struct ExitLatch(mpsc::SyncSender<()>, mpsc::Receiver<()>);
    impl Drop for ExitLatch {
        fn drop(&mut self) {
            let _ = self.0.send(());
            let _ = self.1.recv_timeout(Duration::from_secs(2));
        }
    }
    thread_local! {
        static EXIT_LATCH: RefCell<Option<ExitLatch>> = const { RefCell::new(None) };
    }
    struct TlsOutput(Option<ExitLatch>);
    impl std::io::Write for TlsOutput {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl super::super::worker::DurableOutput for TlsOutput {
        fn sync_data(&mut self) -> std::io::Result<()> {
            if let Some(latch) = self.0.take() {
                EXIT_LATCH.with(|slot| *slot.borrow_mut() = Some(latch));
            }
            Ok(())
        }
    }

    let (entered, entered_receiver) = mpsc::sync_channel(1);
    let (release, release_receiver) = mpsc::sync_channel(1);
    let runtime = SinkRuntime::start(
        Some(Box::new(TlsOutput(Some(ExitLatch(
            entered,
            release_receiver,
        ))))),
        false,
        DiagnosticLogSinkSettings::default(),
    )
    .unwrap();
    let started_at = Instant::now();
    let stopped = runtime.shutdown_for_library_unload(Duration::from_millis(25));
    let elapsed = started_at.elapsed();
    let tls_entered = entered_receiver.recv_timeout(Duration::from_secs(1));
    let retry_stopped = runtime.shutdown_for_library_unload(Duration::ZERO);
    let _ = release.send(());
    let joined = runtime.shutdown_for_library_unload(Duration::from_secs(1));

    assert!(tls_entered.is_ok());
    assert!(!stopped);
    assert!(elapsed < Duration::from_secs(1));
    assert!(!retry_stopped);
    assert!(joined);
    assert!(runtime.shutdown_for_library_unload(Duration::ZERO));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalOutputOperation {
    Write,
    Flush,
    Sync,
}

#[derive(Default)]
struct TerminalOutputProbe {
    calls: [std::sync::atomic::AtomicUsize; 3],
    bytes: std::sync::Mutex<Vec<u8>>,
    dropped: AtomicBool,
}

impl TerminalOutputProbe {
    fn calls(&self) -> [usize; 3] {
        self.calls
            .each_ref()
            .map(|calls| calls.load(Ordering::Acquire))
    }

    fn text(&self) -> String {
        String::from_utf8(self.bytes.lock().unwrap().clone()).unwrap()
    }
}

struct TerminalResultOutput {
    probe: Arc<TerminalOutputProbe>,
    failure: Option<TerminalOutputOperation>,
}

impl TerminalResultOutput {
    fn operation(&self, operation: TerminalOutputOperation) -> std::io::Result<()> {
        self.probe.calls[operation as usize].fetch_add(1, Ordering::Relaxed);
        if self.failure == Some(operation) {
            return Err(std::io::Error::other("injected terminal output failure"));
        }
        Ok(())
    }
}

impl std::io::Write for TerminalResultOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.operation(TerminalOutputOperation::Write)?;
        self.probe.bytes.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.operation(TerminalOutputOperation::Flush)
    }
}

impl super::super::worker::DurableOutput for TerminalResultOutput {
    fn sync_data(&mut self) -> std::io::Result<()> {
        self.operation(TerminalOutputOperation::Sync)
    }
}

impl Drop for TerminalResultOutput {
    fn drop(&mut self) {
        self.probe.dropped.store(true, Ordering::Release);
    }
}

fn terminal_result_test_state(
    probe: Arc<TerminalOutputProbe>,
    failure: Option<TerminalOutputOperation>,
) -> DiagnosticLogState {
    DiagnosticLogState::with_test_sink(
        SinkRuntime::start(
            Some(Box::new(TerminalResultOutput { probe, failure })),
            false,
            DiagnosticLogSinkSettings::default()
                .with_queue_capacity(8)
                .with_max_batch_records(8)
                .with_flush_interval(Duration::from_secs(60)),
        )
        .expect("terminal result sink worker"),
    )
}

#[test]
fn process_shutdown_repeats_completed_worker_output_failures() {
    for (operation, written_records) in [
        (TerminalOutputOperation::Write, 0),
        (TerminalOutputOperation::Flush, 0),
        (TerminalOutputOperation::Sync, 1),
    ] {
        let controller = ProcessLogController::default();
        assert!(controller.shutdown_when_idle(Duration::ZERO));
        let probe = Arc::new(TerminalOutputProbe::default());
        let state = controller.publish_state(
            terminal_result_test_state(Arc::clone(&probe), Some(operation)),
            &mut controller.lock_lifecycle(),
        );
        state.write(DiagnosticLogLevel::Log, "terminal", "failed output");
        let sink = state.sink.as_ref().unwrap();

        assert!(!controller.shutdown_when_idle(Duration::from_secs(2)));
        assert!(controller.active_state_for_test().is_none());
        assert!(probe.dropped.load(Ordering::Acquire));
        assert!(sink.shutdown_for_library_unload(Duration::ZERO));
        assert!(!sink.shutdown(Duration::ZERO));
        let snapshot = sink.snapshot();
        assert!(snapshot.closed);
        assert!(snapshot.output_errors > 0);
        assert_eq!(snapshot.written_records, written_records);
        assert_eq!(snapshot.queue_depth, 0);
        let calls = probe.calls();
        assert!(calls[operation as usize] > 0);

        for _ in 0..3 {
            assert!(!controller.shutdown_when_idle(Duration::ZERO));
            assert!(!controller.shutdown_when_idle(Duration::from_millis(1)));
            assert!(controller.active_state_for_test().is_none());
            assert_eq!(probe.calls(), calls);
            assert_eq!(sink.snapshot().output_errors, snapshot.output_errors);
            assert_eq!(sink.snapshot().written_records, snapshot.written_records);
        }
    }
}

#[test]
fn process_shutdown_repeats_completed_success_without_restarting_worker() {
    let controller = ProcessLogController::default();
    let probe = Arc::new(TerminalOutputProbe::default());
    let state = controller.publish_state(
        terminal_result_test_state(Arc::clone(&probe), None),
        &mut controller.lock_lifecycle(),
    );
    state.write(DiagnosticLogLevel::Log, "terminal", "first record");
    state.write(DiagnosticLogLevel::Log, "terminal", "second record");
    let sink = state.sink.as_ref().unwrap();

    assert!(controller.shutdown_when_idle(Duration::from_secs(2)));
    assert!(controller.active_state_for_test().is_none());
    assert!(probe.dropped.load(Ordering::Acquire));
    let snapshot = sink.snapshot();
    assert!(snapshot.closed);
    assert_eq!(snapshot.output_errors, 0);
    assert_eq!(snapshot.written_records, 2);
    assert_eq!(snapshot.flush_batches, 1);
    assert_eq!(snapshot.queue_depth, 0);
    let text = probe.text();
    assert!(text.find("first record").unwrap() < text.find("second record").unwrap());
    let calls = probe.calls();
    assert!(calls.iter().all(|calls| *calls > 0));

    for _ in 0..3 {
        assert!(controller.shutdown_when_idle(Duration::ZERO));
        assert!(controller.shutdown_when_idle(Duration::from_millis(1)));
        assert!(controller.active_state_for_test().is_none());
        assert_eq!(probe.calls(), calls);
        assert_eq!(sink.snapshot().written_records, 2);
        assert_eq!(sink.snapshot().flush_batches, 1);
    }
    assert_eq!(probe.text(), text);
}

#[test]
fn process_shutdown_terminal_result_resets_only_after_fresh_generation_publication() {
    let controller = ProcessLogController::default();
    let failed_probe = Arc::new(TerminalOutputProbe::default());
    let failed = controller.acquire_dynamic_session_for_test(|| {
        terminal_result_test_state(
            Arc::clone(&failed_probe),
            Some(TerminalOutputOperation::Sync),
        )
    });
    failed.write(DiagnosticLogLevel::Log, "terminal", "old generation");
    // Output failure must not prevent the final dynamic owner from joining and unloading.
    assert!(controller.release_dynamic_session_with_timeout_for_test(Duration::from_secs(2)));
    assert_eq!(controller.dynamic_session_count_for_test(), 0);
    assert!(controller.active_state_for_test().is_none());
    assert!(failed_probe.dropped.load(Ordering::Acquire));
    assert!(failed.sink.as_ref().unwrap().snapshot().output_errors > 0);
    assert!(!controller.shutdown_when_idle(Duration::ZERO));

    let failed_construction = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        controller.acquire_dynamic_session_for_test(|| panic!("generation construction failed"));
    }));
    assert!(failed_construction.is_err());
    assert_eq!(controller.dynamic_session_count_for_test(), 0);
    assert!(controller.active_state_for_test().is_none());
    assert!(!controller.shutdown_when_idle(Duration::ZERO));
    assert_eq!(
        controller.lock_lifecycle().completed_output_result,
        Some(false)
    );

    let fresh_probe = Arc::new(TerminalOutputProbe::default());
    let fresh = controller.acquire_dynamic_session_for_test(|| {
        terminal_result_test_state(Arc::clone(&fresh_probe), None)
    });
    assert!(!Arc::ptr_eq(&failed, &fresh));
    assert!(controller
        .active_state_for_test()
        .is_some_and(|active| Arc::ptr_eq(&active, &fresh)));
    assert_eq!(controller.lock_lifecycle().completed_output_result, None);
    let another = controller.acquire_dynamic_session_for_test(|| {
        panic!("a live generation must not be constructed again")
    });
    assert!(Arc::ptr_eq(&fresh, &another));
    assert_eq!(controller.dynamic_session_count_for_test(), 2);
    fresh.write(DiagnosticLogLevel::Log, "terminal", "fresh generation");
    assert!(controller.release_dynamic_session_with_timeout_for_test(Duration::from_secs(2)));
    assert_eq!(controller.dynamic_session_count_for_test(), 1);
    assert!(fresh.sink.as_ref().unwrap().is_open());
    assert_eq!(controller.lock_lifecycle().completed_output_result, None);

    assert!(controller.release_dynamic_session_with_timeout_for_test(Duration::from_secs(2)));
    assert_eq!(controller.dynamic_session_count_for_test(), 0);
    assert!(controller.active_state_for_test().is_none());
    assert!(fresh_probe.dropped.load(Ordering::Acquire));
    assert_eq!(fresh.sink.as_ref().unwrap().snapshot().output_errors, 0);
    assert_eq!(fresh.sink.as_ref().unwrap().snapshot().written_records, 1);
    assert!(fresh_probe.text().contains("fresh generation"));
    assert_eq!(
        controller.lock_lifecycle().completed_output_result,
        Some(true)
    );
    assert!(controller.shutdown_when_idle(Duration::ZERO));
    assert!(controller.shutdown_when_idle(Duration::from_millis(1)));
}

#[test]
fn process_shutdown_timeout_retains_generation_until_worker_completion() {
    let controller = ProcessLogController::default();
    let output = BlockingOutput::default();
    let state = controller.publish_state(
        DiagnosticLogState::with_test_sink(
            SinkRuntime::start(
                Some(Box::new(output.clone())),
                false,
                DiagnosticLogSinkSettings::default().with_max_batch_records(1),
            )
            .expect("blocked terminal result worker"),
        ),
        &mut controller.lock_lifecycle(),
    );
    let sink = state.sink.as_ref().unwrap();
    assert!(sink.enqueue(DiagnosticLogLevel::Log, "terminal", "timeout record"));
    output.wait_until_blocked();

    let stopped = controller.shutdown_when_idle(Duration::from_millis(20));
    let retained = controller.active_state_for_test();
    let unfinished_result = controller.lock_lifecycle().completed_output_result;
    let closed_before_release = sink.snapshot().closed;
    output.release();
    let joined = controller.shutdown_when_idle(Duration::from_secs(2));

    assert!(!stopped);
    assert!(retained.is_some_and(|active| Arc::ptr_eq(&active, &state)));
    assert_eq!(unfinished_result, None);
    assert!(!closed_before_release);
    assert!(joined);
    assert!(controller.active_state_for_test().is_none());
    assert!(sink.shutdown_for_library_unload(Duration::ZERO));
    assert_eq!(sink.snapshot().written_records, 1);
    assert_eq!(sink.snapshot().output_errors, 0);
    assert_eq!(
        controller.lock_lifecycle().completed_output_result,
        Some(true)
    );
    assert!(output.text().contains("timeout record"));
    assert!(controller.shutdown_when_idle(Duration::ZERO));
    assert!(controller.shutdown_when_idle(Duration::from_millis(1)));
}
