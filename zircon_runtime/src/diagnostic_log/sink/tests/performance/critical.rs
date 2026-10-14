use std::sync::{mpsc, Arc, Barrier};
use std::time::Duration;

use super::super::super::worker::SinkRuntime;
use super::super::backpressure::wait_until_blocked_result;
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
                .with_critical_enqueue_timeout(Duration::from_millis(10))
                .with_flush_interval(Duration::from_secs(60)),
        )
        .expect("critical companion sink"),
    );
    let in_flight_accepted = runtime.enqueue(DiagnosticLogLevel::Debug, "perf", "in-flight");
    let blocking_output = if in_flight_accepted {
        wait_until_blocked_result(&output)
    } else {
        Ok(())
    };
    let queued_accepted =
        in_flight_accepted && runtime.enqueue(DiagnosticLogLevel::Debug, "perf", "queued");

    let (completed_tx, completed_rx) = mpsc::sync_channel(2);
    let start = Arc::new(Barrier::new(3));
    let mut producers = Vec::new();
    for level in [DiagnosticLogLevel::Warn, DiagnosticLogLevel::Error] {
        let runtime = Arc::clone(&runtime);
        let completed_tx = completed_tx.clone();
        let start = Arc::clone(&start);
        producers.push(std::thread::spawn(move || {
            start.wait();
            completed_tx.send((level, runtime.enqueue(level, "perf", "critical")))
        }));
    }
    drop(completed_tx);
    start.wait();
    let completed_results = [
        completed_rx.recv_timeout(Duration::from_millis(250)),
        completed_rx.recv_timeout(Duration::from_millis(250)),
    ];
    output.release();
    let producer_results: Vec<_> = producers
        .into_iter()
        .map(|producer| producer.join())
        .collect();
    let shutdown = runtime.shutdown(Duration::from_secs(2));

    assert!(in_flight_accepted);
    blocking_output.expect("critical companion worker enters the blocking output");
    assert!(queued_accepted);
    for result in producer_results {
        result
            .expect("critical companion producer")
            .expect("critical companion completion receiver");
    }
    let completed = completed_results.map(|result| {
        result.expect("critical companion producer must return before output release")
    });
    assert!(completed.iter().all(|(_, accepted)| !*accepted));
    assert!(shutdown);
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.dropped_warn, 1);
    assert_eq!(snapshot.dropped_error, 1);
    assert_eq!(snapshot.critical_backpressure_count, 2);
    assert_eq!(snapshot.written_records, 2);
    let text = output.text();
    assert_eq!(text.lines().count(), 2);
    assert!(!text.contains("[warn] [perf] critical"));
    assert!(!text.contains("[error] [perf] critical"));
    assert!(text.contains("[debug] [perf] in-flight"));
    assert!(text.contains("[debug] [perf] queued"));
    println!(
        "PERF-MVP-434 bounded_critical_timeout_ms=10 critical_backpressure={} dropped_warn={} dropped_error={} written={}",
        snapshot.critical_backpressure_count,
        snapshot.dropped_warn,
        snapshot.dropped_error,
        snapshot.written_records
    );
}
