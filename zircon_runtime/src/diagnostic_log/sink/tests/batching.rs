use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::super::worker::SinkRuntime;
use super::fixtures::SharedOutput;
use crate::diagnostic_log::{DiagnosticLogLevel, DiagnosticLogSinkSettings};

#[test]
fn continuous_nonempty_queue_flushes_at_the_time_threshold() {
    let output = SharedOutput::default();
    let runtime = Arc::new(
        SinkRuntime::start(
            Some(Box::new(output.clone())),
            false,
            DiagnosticLogSinkSettings::default()
                .with_queue_capacity(4_096)
                .with_max_batch_records(usize::MAX)
                .with_max_batch_bytes(usize::MAX)
                .with_flush_interval(Duration::from_millis(20)),
        )
        .expect("sink worker"),
    );
    let stop = Arc::new(AtomicBool::new(false));
    let producer_runtime = Arc::clone(&runtime);
    let producer_stop = Arc::clone(&stop);
    let producer = std::thread::spawn(move || {
        while !producer_stop.load(Ordering::Acquire) {
            producer_runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "stream");
        }
    });

    let deadline = Instant::now() + Duration::from_millis(500);
    while output.text().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        output.text().contains("stream"),
        "continuous input must not postpone the time-based flush"
    );

    stop.store(true, Ordering::Release);
    producer.join().unwrap();
    assert!(runtime.shutdown(Duration::from_secs(2)));
}

#[test]
fn byte_threshold_flushes_before_accepting_the_next_batch_record() {
    let output = SharedOutput::default();
    let runtime = SinkRuntime::start(
        Some(Box::new(output.clone())),
        false,
        DiagnosticLogSinkSettings::default()
            .with_max_batch_records(16)
            .with_max_batch_bytes(100)
            .with_flush_interval(Duration::from_secs(60)),
    )
    .expect("sink worker");

    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "first"));
    assert!(runtime.enqueue(DiagnosticLogLevel::Log, "runtime", "second"));
    let deadline = Instant::now() + Duration::from_secs(1);
    while output.text().is_empty() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(output.text().contains("first"));
    assert!(!output.text().contains("second"));

    assert!(runtime.shutdown(Duration::from_secs(2)));
    assert!(output.text().contains("second"));
}
