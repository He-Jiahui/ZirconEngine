use std::cell::Cell;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use super::{DiagnosticsShard, JobSchedulerDiagnosticsState, DIAGNOSTIC_SHARD_COUNT};

#[test]
fn terminal_observation_source_does_not_enable_full_lifecycle_sampling() {
    let state = JobSchedulerDiagnosticsState::default();
    let source = state.task_diagnostic_source();
    let cursor = source.initial_cursor();

    assert!(state.record_scheduled_and_enqueued().is_none());
    let identity = state
        .task_identity()
        .expect("the terminal observation source should allocate task identity");
    state.record_task_observation(
        Some(identity),
        super::TaskDiagnosticKind::Cancelled,
        Arc::from("observation-only cancellation"),
    );

    assert_eq!(state.report().scheduled, 0);
    assert_eq!(source.read_after(cursor, 1).observations().len(), 1);
}

#[test]
fn task_identity_sequences_are_unique_without_a_scheduler_global_allocator() {
    let shards: [DiagnosticsShard; DIAGNOSTIC_SHARD_COUNT] =
        std::array::from_fn(DiagnosticsShard::for_index);
    let mut sequences = Vec::with_capacity(DIAGNOSTIC_SHARD_COUNT * 2);
    for shard in &shards {
        sequences.push(shard.allocate_task_sequence());
        sequences.push(shard.allocate_task_sequence());
    }
    sequences.sort_unstable();
    sequences.dedup();

    assert_eq!(sequences.len(), DIAGNOSTIC_SHARD_COUNT * 2);
    assert_eq!(sequences[0], 1);
    assert_eq!(sequences[DIAGNOSTIC_SHARD_COUNT * 2 - 1], 128);
}

#[test]
fn disabled_diagnostics_do_not_allocate_lifecycle_samples() {
    let state = JobSchedulerDiagnosticsState::default();

    assert!(state.record_scheduled_and_enqueued().is_none());
    assert_eq!(state.report().scheduled, 0);

    state.enable();
    let enqueued_at = state
        .record_scheduled_and_enqueued()
        .expect("enabled diagnostics should record queue admission");
    assert!(state.record_started(Some(enqueued_at)));
    state.record_active_terminal(false, state.execution_started_at(true));

    let report = state.report();
    assert_eq!(report.scheduled, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.execution_samples, 1);
    assert!(report.execution_ms >= 0.0);
}

#[test]
fn cancelled_task_without_worker_start_has_no_execution_sample() {
    let state = JobSchedulerDiagnosticsState::default();
    state.enable();

    assert!(state.record_scheduled());
    state.record_cancelled(true);

    let report = state.report();
    assert_eq!(report.scheduled, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.execution_samples, 0);
    assert_eq!(report.execution_ms, 0.0);
}

#[test]
fn cancelled_task_after_worker_start_retires_active_and_records_execution() {
    let state = JobSchedulerDiagnosticsState::default();
    state.enable();

    let enqueued_at = state
        .record_scheduled_and_enqueued()
        .expect("enabled diagnostics should record queue admission");
    assert!(state.record_started(Some(enqueued_at)));
    state.record_active_cancelled(state.execution_started_at(true));

    let report = state.report();
    assert_eq!(report.scheduled, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.active, 0);
    assert_eq!(report.execution_samples, 1);
}

#[test]
fn overlapping_diagnostic_writers_publish_one_stable_lifecycle_snapshot() {
    let state = Arc::new(JobSchedulerDiagnosticsState::default());
    state.enable();
    let entered = Arc::new(Barrier::new(3));
    let release = Arc::new(Barrier::new(3));
    let mut writers = Vec::new();

    for _ in 0..2 {
        let state = Arc::clone(&state);
        let entered = Arc::clone(&entered);
        let release = Arc::clone(&release);
        writers.push(thread::spawn(move || {
            let shard = &state.shards[0];
            let _update = shard.begin_update();
            shard
                .scheduled
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            shard
                .enqueued
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            entered.wait();
            release.wait();
        }));
    }

    entered.wait();
    release.wait();
    for writer in writers {
        writer.join().unwrap();
    }

    let report = state.report();
    assert_eq!(report.scheduled, 2);
    assert_eq!(report.queued, 2);
    assert_eq!(report.dependency_waiting, 0);

    let source = include_str!("../diagnostics.rs");
    assert!(source.contains("const DIAGNOSTIC_SHARD_COUNT: usize = 64"));
    assert!(source.contains("#[repr(align(64))]"));
}

#[test]
fn aggregate_snapshot_retries_after_transient_shard_contention() {
    let state = JobSchedulerDiagnosticsState::default();
    let shard = &state.shards[0];
    shard
        .scheduled
        .store(1, std::sync::atomic::Ordering::Relaxed);
    shard
        .enqueued
        .store(1, std::sync::atomic::Ordering::Relaxed);
    shard
        .updates_in_flight
        .store(1, std::sync::atomic::Ordering::Release);
    assert!(state.try_stable_snapshot_attempt().is_none());
    let attempts = Cell::new(0);

    let snapshot = state
        .try_stable_snapshot_with_attempt_hook(|attempt| {
            attempts.set(attempt + 1);
            if attempt == 1 {
                shard
                    .updates_in_flight
                    .store(0, std::sync::atomic::Ordering::Release);
            }
        })
        .expect("the aggregate retry budget should outlive transient shard contention");

    assert_eq!(attempts.get(), 2);
    let report = snapshot.report();
    assert_eq!(report.scheduled, 1);
    assert_eq!(report.queued, 1);
}

#[test]
#[ignore = "managed release benchmark"]
fn aggregate_snapshot_retry_benchmark() {
    const SAMPLE_COUNT: usize = 101;
    let state = JobSchedulerDiagnosticsState::default();
    let shard = &state.shards[0];
    shard
        .scheduled
        .store(1, std::sync::atomic::Ordering::Relaxed);
    shard
        .enqueued
        .store(1, std::sync::atomic::Ordering::Relaxed);
    let mut retired_samples_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut retired_fresh_snapshots = 0;
    let mut optimized_fresh_snapshots = 0;

    for _ in 0..SAMPLE_COUNT {
        shard
            .updates_in_flight
            .store(1, std::sync::atomic::Ordering::Release);
        let retired_started = Instant::now();
        retired_fresh_snapshots += usize::from(state.try_stable_snapshot_attempt().is_some());
        retired_samples_ns.push(retired_started.elapsed().as_nanos());

        let started = Instant::now();
        let snapshot = state
            .try_stable_snapshot_with_attempt_hook(|attempt| {
                if attempt == 1 {
                    shard
                        .updates_in_flight
                        .store(0, std::sync::atomic::Ordering::Release);
                }
            })
            .expect("the second aggregate attempt should publish a fresh snapshot");
        optimized_samples_ns.push(started.elapsed().as_nanos());
        optimized_fresh_snapshots += 1;
        assert_eq!(snapshot.report().scheduled, 1);
    }

    retired_samples_ns.sort_unstable();
    optimized_samples_ns.sort_unstable();
    let p95_index = (SAMPLE_COUNT * 95).div_ceil(100) - 1;
    let retired_p95_ns = retired_samples_ns[p95_index];
    let optimized_p95_ns = optimized_samples_ns[p95_index];
    eprintln!(
        "TASK_DIAGNOSTICS_SNAPSHOT_RETRY samples={SAMPLE_COUNT} retired_fresh_snapshots={retired_fresh_snapshots} optimized_fresh_snapshots={optimized_fresh_snapshots} aggregate_attempts=2 retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns}"
    );
    assert_eq!(retired_fresh_snapshots, 0);
    assert_eq!(optimized_fresh_snapshots, SAMPLE_COUNT);
    assert!(
        optimized_p95_ns <= 2_000_000,
        "two-attempt aggregate snapshot P95 must stay within 2 ms"
    );
}
