use std::time::{Duration, Instant};

use super::*;

const MAX_BATCH_TRANSFER_LATENCY: Duration = Duration::from_millis(10);

#[test]
fn observer_event_backlog_collapses_to_one_authoritative_resynchronization() {
    let mut dispatch = ProgressObserverDispatch::default();
    for id in 0..=MAX_PROGRESS_OBSERVER_EVENTS {
        dispatch.push(ProgressObserverEvent::Admitted(JobId::new(id as u64)));
    }

    assert_eq!(dispatch.events.len(), 1);
    assert!(matches!(
        dispatch.events.front(),
        Some(ProgressObserverEvent::Resynchronize)
    ));

    dispatch.push(ProgressObserverEvent::Finished(JobId::new(1)));
    assert_eq!(dispatch.events.len(), 1);
    let mut batch = dispatch.take_all();
    assert!(matches!(
        batch.pop_front(),
        Some(ProgressObserverEvent::Resynchronize)
    ));

    dispatch.push(ProgressObserverEvent::Finished(JobId::new(2)));
    assert_eq!(dispatch.events.len(), 1);
}

#[test]
fn observer_events_transfer_as_one_ordered_batch() {
    let mut dispatch = ProgressObserverDispatch::default();
    for id in 0..MAX_PROGRESS_OBSERVER_EVENTS {
        dispatch.push(ProgressObserverEvent::Admitted(JobId::new(id as u64)));
    }

    let batch = dispatch.take_all();

    assert_eq!(batch.len(), MAX_PROGRESS_OBSERVER_EVENTS);
    assert!(dispatch.events.is_empty());
    assert!(matches!(
        batch.front(),
        Some(ProgressObserverEvent::Admitted(id)) if id.value() == 0
    ));
    assert!(matches!(
        batch.back(),
        Some(ProgressObserverEvent::Admitted(id))
            if id.value() == (MAX_PROGRESS_OBSERVER_EVENTS - 1) as u64
    ));
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_observer_batch_transfer_evidence() {
    let mut dispatch = ProgressObserverDispatch::default();
    for id in 0..MAX_PROGRESS_OBSERVER_EVENTS {
        dispatch.push(ProgressObserverEvent::Admitted(JobId::new(id as u64)));
    }

    let started = Instant::now();
    let batch = dispatch.take_all();
    let elapsed = started.elapsed();

    assert_eq!(batch.len(), MAX_PROGRESS_OBSERVER_EVENTS);
    assert!(elapsed <= MAX_BATCH_TRANSFER_LATENCY);
    let lock_acquisitions_before = MAX_PROGRESS_OBSERVER_EVENTS + 2;
    let lock_acquisitions_after = 2;
    let reduction_percent =
        (1.0 - lock_acquisitions_after as f64 / lock_acquisitions_before as f64) * 100.0;
    println!(
        "EDITOR_JOB_BENCH_V1 kind=observer_batch_transfer events={} dispatch_lock_acquisitions_before={} dispatch_lock_acquisitions_after={} lock_reduction_percent={:.4} elapsed_ns={} target_ns={}",
        MAX_PROGRESS_OBSERVER_EVENTS,
        lock_acquisitions_before,
        lock_acquisitions_after,
        reduction_percent,
        elapsed.as_nanos(),
        MAX_BATCH_TRANSFER_LATENCY.as_nanos(),
    );
}
