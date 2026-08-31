use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

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
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "in-flight"));
    writer.wait_until_blocked();
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "queued"));

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for index in 0..SAMPLE_COUNT {
        let level = if index % 2 == 0 {
            DiagnosticLogLevel::Warn
        } else {
            DiagnosticLogLevel::Error
        };
        let started = Instant::now();
        assert!(!runtime.enqueue(level, "runtime", "critical"));
        samples.push(started.elapsed().as_nanos());
    }

    writer.release();
    assert!(runtime.shutdown(Duration::from_secs(2)));
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
