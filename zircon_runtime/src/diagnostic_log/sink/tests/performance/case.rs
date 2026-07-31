use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Barrier, OnceLock};
use std::time::{Duration, Instant};

use super::configuration::{active_scope, test_state, QUEUE_CAPACITY};
use super::output::InstrumentedSlowOutput;
use super::pacing::{event_offset, wait_until, LOAD_WINDOW};
use super::report::{assert_case, percentile_95, CaseReport};
use super::resources::CaseResources;
use crate::diagnostic_log::DiagnosticLogLevel;

pub(super) fn run_case(
    logs_per_second: usize,
    caller_count: usize,
    scoped_rule_count: usize,
    sink_delay: Duration,
) -> CaseReport {
    let output = InstrumentedSlowOutput::new(sink_delay, logs_per_second);
    let resources = CaseResources::new(test_state(scoped_rule_count, output.clone()));
    let state = resources.state();
    let formatted = Arc::new(AtomicUsize::new(0));
    let cancel = Arc::new(AtomicBool::new(false));
    let ready = Arc::new(Barrier::new(caller_count + 1));
    let start_time = Arc::new(OnceLock::new());
    let scope = Arc::new(active_scope(scoped_rule_count));
    let mut callers = Vec::with_capacity(caller_count);
    let (completed_tx, completed_rx) = mpsc::sync_channel(caller_count);

    for caller_index in 0..caller_count {
        let state = Arc::clone(&state);
        let formatted = Arc::clone(&formatted);
        let cancel = Arc::clone(&cancel);
        let ready = Arc::clone(&ready);
        let start_time = Arc::clone(&start_time);
        let scope = Arc::clone(&scope);
        let completed_tx = completed_tx.clone();
        callers.push(std::thread::spawn(move || {
            ready.wait();
            let load_started = *start_time.wait();
            let mut latencies = Vec::new();
            for sequence in (caller_index..logs_per_second).step_by(caller_count) {
                if cancel.load(Ordering::Acquire) {
                    break;
                }
                wait_until(load_started + event_offset(sequence, logs_per_second));
                let call_started = Instant::now();
                state.write_lazy(DiagnosticLogLevel::Debug, scope.as_str(), || {
                    formatted.fetch_add(1, Ordering::Relaxed);
                    format!("record={sequence} caller={caller_index}")
                });
                latencies.push(call_started.elapsed());
            }
            completed_tx.send(latencies).unwrap();
        }));
    }
    drop(completed_tx);
    ready.wait();
    let load_started = Instant::now() + Duration::from_millis(50);
    start_time
        .set(load_started)
        .expect("load start published once");
    let caller_deadline = load_started + Duration::from_millis(1_250);
    let mut latencies = Vec::new();
    for _ in 0..caller_count {
        match completed_rx.recv_timeout(caller_deadline.saturating_duration_since(Instant::now())) {
            Ok(mut caller_latencies) => latencies.append(&mut caller_latencies),
            Err(error) => {
                cancel.store(true, Ordering::Release);
                panic!("log storm caller exceeded deadline: {error}");
            }
        }
    }
    for caller in callers {
        caller.join().expect("completed log storm caller");
    }
    wait_until(load_started + LOAD_WINDOW);
    let load_elapsed = load_started.elapsed();

    let (sink, rss) = resources.finish();
    let output = output.snapshot();
    assert_case(
        logs_per_second,
        formatted.load(Ordering::Relaxed),
        QUEUE_CAPACITY,
        sink_delay,
        load_elapsed,
        &sink,
        &output,
        rss,
    );
    latencies.sort_unstable();

    CaseReport::new(
        logs_per_second,
        caller_count,
        scoped_rule_count,
        sink_delay,
        percentile_95(&latencies),
        load_elapsed,
        rss,
        sink,
        output,
    )
}
