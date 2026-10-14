use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use super::super::worker::SinkRuntime;
use super::super::DiagnosticLogState;
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
fn full_queue_does_not_evaluate_best_effort_lazy_message() {
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

    let evaluated = AtomicBool::new(false);
    assert!(
        !runtime.enqueue_lazy(DiagnosticLogLevel::Debug, "runtime", || {
            evaluated.store(true, Ordering::Relaxed);
            "dropped"
        })
    );
    assert!(!evaluated.load(Ordering::Relaxed));
    assert_eq!(runtime.snapshot().dropped_debug, 1);

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
fn full_state_queue_skips_best_effort_lazy_formatting() {
    let writer = BlockingOutput::default();
    let state = DiagnosticLogState::with_test_sink(
        SinkRuntime::start(
            Some(Box::new(writer.clone())),
            false,
            one_record_blocking_settings(),
        )
        .expect("state sink worker"),
    );
    let formatted = AtomicUsize::new(0);
    state.write_lazy(DiagnosticLogLevel::Debug, "runtime", || {
        formatted.fetch_add(1, Ordering::Relaxed);
        "in-flight"
    });
    let blocking_output = wait_until_blocked_result(&writer);
    state.write_lazy(DiagnosticLogLevel::Debug, "runtime", || {
        formatted.fetch_add(1, Ordering::Relaxed);
        "queued"
    });
    state.write_lazy(DiagnosticLogLevel::Debug, "runtime", || {
        formatted.fetch_add(1, Ordering::Relaxed);
        "dropped"
    });
    let formatted_before_release = formatted.load(Ordering::Relaxed);
    let held_snapshot = state.snapshot();

    writer.release();
    let shutdown = state
        .sink
        .as_ref()
        .expect("state sink runtime")
        .shutdown(Duration::from_secs(2));
    let snapshot = state.snapshot().expect("state sink snapshot");

    blocking_output.expect("state sink worker enters the blocking output");
    assert!(shutdown);
    assert_eq!(formatted_before_release, 2);
    assert_eq!(held_snapshot.expect("held sink snapshot").queue_depth, 1);
    assert_eq!(snapshot.dequeued_records, 2);
    assert_eq!(snapshot.written_records, 2);
    assert_eq!(snapshot.dropped_debug, 1);
    assert_eq!(snapshot.dequeued_records + snapshot.dropped_debug, 3);
    assert_eq!(snapshot.queue_depth, 0);
    assert!(snapshot.closed);
    assert_eq!(snapshot.output_errors, 0);
    let text = writer.text();
    assert_eq!(text.lines().count(), 2);
    assert!(text.contains("[debug] [runtime] in-flight"));
    assert!(text.contains("[debug] [runtime] queued"));
    assert!(!text.contains("[debug] [runtime] dropped"));
}

#[test]
fn state_lazy_formatting_can_lose_a_concurrent_queue_admission() {
    let writer = BlockingOutput::default();
    let state = Arc::new(DiagnosticLogState::with_test_sink(
        SinkRuntime::start(
            Some(Box::new(writer.clone())),
            false,
            one_record_blocking_settings(),
        )
        .expect("state sink worker"),
    ));
    let formatted = Arc::new(AtomicUsize::new(0));
    state.write_lazy(DiagnosticLogLevel::Debug, "runtime", || {
        formatted.fetch_add(1, Ordering::Relaxed);
        "in-flight"
    });
    let blocking_output = wait_until_blocked_result(&writer);

    let (formatting_tx, formatting_rx) = mpsc::sync_channel(1);
    let (resume_tx, resume_rx) = mpsc::sync_channel(1);
    let (completed_tx, completed_rx) = mpsc::sync_channel(1);
    let producer_state = Arc::clone(&state);
    let producer_formatted = Arc::clone(&formatted);
    let producer = std::thread::spawn(move || {
        let mut formatter_started = false;
        let mut formatter_resumed = false;
        producer_state.write_lazy(DiagnosticLogLevel::Debug, "runtime", || {
            producer_formatted.fetch_add(1, Ordering::Relaxed);
            formatter_started = formatting_tx.send(()).is_ok();
            formatter_resumed = resume_rx.recv().is_ok();
            "dropped"
        });
        let outcome = (formatter_started, formatter_resumed);
        let _ = completed_tx.send(outcome);
        outcome
    });
    let formatting_started = formatting_rx.recv_timeout(Duration::from_secs(2));
    if formatting_started.is_ok() {
        state.write_lazy(DiagnosticLogLevel::Debug, "runtime", || {
            formatted.fetch_add(1, Ordering::Relaxed);
            "winner"
        });
    }
    let resumed = resume_tx.send(());
    let completed_while_held = completed_rx.recv_timeout(Duration::from_secs(2));
    let held_snapshot = state.snapshot();

    writer.release();
    let producer_result = producer.join();
    let shutdown = state
        .sink
        .as_ref()
        .expect("state sink runtime")
        .shutdown(Duration::from_secs(2));
    let snapshot = state.snapshot().expect("state sink snapshot");

    blocking_output.expect("state sink worker enters the blocking output");
    formatting_started.expect("formatter begins while the queue still has capacity");
    resumed.expect("formatter resume receiver");
    assert_eq!(
        completed_while_held.expect("best-effort admission returns while the writer is held"),
        (true, true)
    );
    assert_eq!(producer_result.expect("state lazy producer"), (true, true));
    assert!(shutdown);
    assert_eq!(formatted.load(Ordering::Relaxed), 3);
    let held_snapshot = held_snapshot.expect("held sink snapshot");
    assert_eq!(held_snapshot.queue_depth, 1);
    assert_eq!(held_snapshot.dropped_debug, 1);
    assert_eq!(snapshot.dequeued_records, 2);
    assert_eq!(snapshot.written_records, 2);
    assert_eq!(snapshot.dropped_debug, 1);
    assert_eq!(snapshot.dequeued_records + snapshot.dropped_debug, 3);
    assert_eq!(snapshot.queue_depth, 0);
    assert!(snapshot.closed);
    assert_eq!(snapshot.output_errors, 0);
    let text = writer.text();
    assert_eq!(text.lines().count(), 2);
    assert!(text.contains("[debug] [runtime] in-flight"));
    assert!(text.contains("[debug] [runtime] winner"));
    assert!(!text.contains("[debug] [runtime] dropped"));
}

#[test]
fn full_queue_bounds_critical_producer_wait_and_records_drops() {
    let writer = BlockingOutput::default();
    let runtime = Arc::new(
        SinkRuntime::start(
            Some(Box::new(writer.clone())),
            false,
            one_record_blocking_settings().with_critical_enqueue_timeout(Duration::from_millis(10)),
        )
        .expect("sink worker"),
    );

    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "in-flight"));
    writer.wait_until_blocked();
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "queued"));

    let (completed_tx, completed_rx) = mpsc::sync_channel(2);
    let mut producers = Vec::new();
    for level in [DiagnosticLogLevel::Warn, DiagnosticLogLevel::Error] {
        let completed_tx = completed_tx.clone();
        let producer_runtime = Arc::clone(&runtime);
        producers.push(std::thread::spawn(move || {
            let started = Instant::now();
            let accepted = producer_runtime.enqueue(level, "runtime", "critical");
            completed_tx
                .send((level, accepted, started.elapsed()))
                .unwrap();
        }));
    }

    let results = [
        completed_rx.recv_timeout(Duration::from_millis(250)),
        completed_rx.recv_timeout(Duration::from_millis(250)),
    ];

    writer.release();
    for producer in producers {
        producer.join().unwrap();
    }
    assert!(runtime.shutdown(Duration::from_secs(2)));

    let completed = results.map(|result| {
        result.expect("critical producer must return after its bounded enqueue timeout")
    });
    assert!(completed.iter().all(|(_, accepted, _)| !accepted));
    assert!(completed
        .iter()
        .all(|(_, _, elapsed)| *elapsed < Duration::from_millis(250)));
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.dropped_warn, 1);
    assert_eq!(snapshot.dropped_error, 1);
    assert_eq!(snapshot.critical_backpressure_count, 2);
    assert!(!writer.text().contains("critical"));
}

#[test]
#[ignore = "managed critical log admission performance gate"]
fn critical_admission_timeout_release_benchmark_evidence() {
    const SAMPLE_COUNT: usize = 20;
    const TIMEOUT: Duration = Duration::from_millis(2);

    let writer = BlockingOutput::default();
    let runtime = SinkRuntime::start(
        Some(Box::new(writer.clone())),
        false,
        one_record_blocking_settings().with_critical_enqueue_timeout(TIMEOUT),
    )
    .expect("sink worker");
    let in_flight_accepted = runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "in-flight");
    let blocking_output = if in_flight_accepted {
        wait_until_blocked_result(&writer)
    } else {
        Ok(())
    };
    let queued_accepted =
        in_flight_accepted && runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "queued");

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut accepted_samples = Vec::with_capacity(SAMPLE_COUNT);
    for index in 0..SAMPLE_COUNT {
        let level = if index % 2 == 0 {
            DiagnosticLogLevel::Warn
        } else {
            DiagnosticLogLevel::Error
        };
        let started = Instant::now();
        let accepted = runtime.enqueue(level, "runtime", "critical");
        let elapsed = started.elapsed().as_nanos();
        samples.push(elapsed);
        accepted_samples.push(accepted);
    }

    writer.release();
    let shutdown = runtime.shutdown(Duration::from_secs(2));
    assert!(in_flight_accepted);
    blocking_output.expect("critical benchmark worker enters the blocking output");
    assert!(queued_accepted);
    assert!(accepted_samples.iter().all(|accepted| !accepted));
    assert!(shutdown);
    let p50 = nearest_rank_percentile(&samples, 50);
    let p95 = nearest_rank_percentile(&samples, 95);
    let max = samples.iter().copied().max().expect("admission samples");
    let admission_ns = samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.dropped_warn, 10);
    assert_eq!(snapshot.dropped_error, 10);
    assert_eq!(snapshot.critical_backpressure_count, SAMPLE_COUNT as u64);
    assert!(p95 <= Duration::from_millis(50).as_nanos());
    println!(
        "CRITICAL_LOG_ADMISSION_BENCH_V1 samples={SAMPLE_COUNT} timeout_ns={} p50_ns={p50} p95_ns={p95} max_ns={max} dropped_warn={} dropped_error={} backpressure={} admission_ns={admission_ns}",
        TIMEOUT.as_nanos(),
        snapshot.dropped_warn,
        snapshot.dropped_error,
        snapshot.critical_backpressure_count
    );
}

pub(super) fn wait_until_blocked_result(output: &BlockingOutput) -> std::thread::Result<()> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| output.wait_until_blocked()))
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn one_record_blocking_settings() -> DiagnosticLogSinkSettings {
    DiagnosticLogSinkSettings::default()
        .with_queue_capacity(1)
        .with_max_batch_records(1)
        .with_flush_interval(Duration::from_secs(60))
}
