//! 通过持续输入和字节阈值验证 worker 的时间/容量双重 flush 边界不会被后续记录推迟。
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

#[test]
fn batch_encoding_preserves_multirecord_bytes_and_order() {
    let output = SharedOutput::default();
    let runtime = SinkRuntime::start(
        Some(Box::new(output.clone())),
        false,
        DiagnosticLogSinkSettings::default()
            .with_queue_capacity(16)
            .with_max_batch_records(16)
            .with_max_batch_bytes(usize::MAX)
            .with_flush_interval(Duration::from_secs(60)),
    )
    .expect("sink worker");
    let records = [
        (DiagnosticLogLevel::Log, "runtime", "first"),
        (
            DiagnosticLogLevel::Warn,
            "path",
            r"literal \n and C:\assets\demo",
        ),
        (DiagnosticLogLevel::Error, "场景", "节点 🧭\n第二行"),
        (DiagnosticLogLevel::Debug, "runtime", "last"),
    ];
    for (level, scope, message) in &records {
        assert!(runtime.enqueue(*level, scope, message));
    }
    assert!(runtime.flush(Duration::from_secs(2)));
    assert!(runtime.shutdown(Duration::from_secs(2)));

    let text = output.text();
    let (timestamp_prefix, _) = text.split_once("] ").expect("batch timestamp prefix");
    let expected = format!(
        "{timestamp_prefix}] [log] [runtime] first\n\
         {timestamp_prefix}] [warn] [path] literal \\n and C:\\assets\\demo\n\
         {timestamp_prefix}] [error] [场景] 节点 🧭\\n第二行\n\
         {timestamp_prefix}] [debug] [runtime] last\n"
    );
    assert_eq!(text, expected);

    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.dequeued_records, records.len() as u64);
    assert_eq!(snapshot.written_records, records.len() as u64);
    assert_eq!(snapshot.written_bytes, text.len() as u64);
    assert_eq!(snapshot.flush_batches, 1);
    assert_eq!(snapshot.queue_depth, 0);
    assert_eq!(snapshot.output_errors, 0);
    assert!(snapshot.closed);
}
