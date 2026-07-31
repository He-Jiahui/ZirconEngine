use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};

use super::super::super::worker::SinkRuntime;
use super::super::fixtures::BlockingOutput;
use crate::diagnostic_log::{DiagnosticLogLevel, DiagnosticLogSinkSettings};

pub(super) fn run_critical_backpressure_companion() {
    let output = BlockingOutput::default();
    let runtime = Arc::new(
        SinkRuntime::start(
            Some(Box::new(output.clone())),
            false,
            DiagnosticLogSinkSettings::default()
                .with_queue_capacity(1)
                .with_max_batch_records(1)
                .with_flush_interval(Duration::from_secs(60)),
        )
        .expect("critical companion sink"),
    );
    assert!(runtime.enqueue(DiagnosticLogLevel::Debug, "perf", "in-flight"));
    output.wait_until_blocked();
    assert!(runtime.enqueue(DiagnosticLogLevel::Debug, "perf", "queued"));

    let (completed_tx, completed_rx) = mpsc::sync_channel(2);
    let start = Arc::new(Barrier::new(3));
    let mut producers = Vec::new();
    for level in [DiagnosticLogLevel::Warn, DiagnosticLogLevel::Error] {
        let runtime = Arc::clone(&runtime);
        let completed_tx = completed_tx.clone();
        let start = Arc::clone(&start);
        producers.push(std::thread::spawn(move || {
            start.wait();
            completed_tx
                .send((level, runtime.enqueue(level, "perf", "critical")))
                .unwrap();
        }));
    }
    start.wait();
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.snapshot().critical_backpressure_count < 2 {
        assert!(
            Instant::now() < deadline,
            "critical producers did not block"
        );
        std::thread::yield_now();
    }
    assert!(matches!(
        completed_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    output.release();
    let completed = [
        completed_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        completed_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
    ];
    for producer in producers {
        producer.join().unwrap();
    }
    assert!(completed.iter().all(|(_, accepted)| *accepted));
    assert!(runtime.shutdown(Duration::from_secs(2)));
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.dropped_warn, 0);
    assert_eq!(snapshot.dropped_error, 0);
    assert!(snapshot.critical_backpressure_count >= 2);
    assert_eq!(snapshot.written_records, 4);
    let text = output.text();
    assert_eq!(text.lines().count(), 4);
    assert!(text.contains("[warn] [perf] critical"));
    assert!(text.contains("[error] [perf] critical"));
    assert!(text.contains("[debug] [perf] in-flight"));
    assert!(text.contains("[debug] [perf] queued"));
    println!(
        "PERF-MVP-434 critical_backpressure={} dropped_warn={} dropped_error={} written={}",
        snapshot.critical_backpressure_count,
        snapshot.dropped_warn,
        snapshot.dropped_error,
        snapshot.written_records
    );
}
