use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use super::*;

fn legacy_combine(handles: &[JobHandle]) -> JobHandle {
    if handles.is_empty() {
        return JobHandle::completed();
    }
    let combined = JobHandle::pending_with_wait_diagnostics(
        handles.len(),
        handles
            .iter()
            .find_map(|handle| handle.wait_diagnostics.clone()),
        handles[0].callback_dispatcher.clone(),
    );
    for handle in handles {
        let handle_for_callback = handle.clone();
        let combined_for_callback = combined.clone();
        let callback = Box::new(move || {
            combined_for_callback.combined_dependency_completed(
                handle_for_callback.panic_message(),
                handle_for_callback.is_cancelled(),
            );
        });
        if !handle.add_dependent(callback) {
            combined.combined_dependency_completed(handle.panic_message(), handle.is_cancelled());
        }
    }
    combined
}

#[test]
fn astra_m24_terminal_outcomes_match_previous_combine() {
    for count in [0, 1, 1_000, 10_000] {
        for states in [
            &[TaskState::Completed][..],
            &[TaskState::Cancelled, TaskState::Completed][..],
            &[
                TaskState::Failed,
                TaskState::Cancelled,
                TaskState::Completed,
            ][..],
        ] {
            let handles = (0..count)
                .map(|index| {
                    let handle = JobHandle::pending_with_dependencies(0);
                    match states[index % states.len()] {
                        TaskState::Completed => handle.mark_complete(),
                        TaskState::Cancelled => handle.mark_cancelled(),
                        TaskState::Failed => {
                            handle.mark_panicked(Arc::from(format!("failure {index}")))
                        }
                        _ => unreachable!(),
                    }
                    handle
                })
                .collect::<Vec<_>>();
            let expected = legacy_combine(&handles);
            let actual = JobHandle::combine(&handles);
            assert!(actual.is_complete());
            assert_eq!(actual.terminal_state(), expected.terminal_state());
            assert_eq!(actual.panic_message(), expected.panic_message());
            assert_eq!(actual.is_cancelled(), expected.is_cancelled());
        }
    }
}

#[test]
fn astra_m24_failure_still_waits_for_duplicate_pending_inputs() {
    let failed = JobHandle::pending_with_dependencies(0);
    failed.mark_panicked(Arc::from("first failure"));
    let cancelled = JobHandle::pending_with_dependencies(0);
    let pending = JobHandle::pending_with_dependencies(0);
    let combined =
        JobHandle::combine(&[failed, cancelled.clone(), pending.clone(), pending.clone()]);
    let observed = Arc::new(AtomicUsize::new(0));
    let observed_on_terminal = Arc::clone(&observed);
    combined.on_terminal(move || {
        observed_on_terminal.fetch_add(1, Ordering::SeqCst);
    });
    assert!(!combined.is_complete());
    cancelled.mark_cancelled();
    assert!(!combined.is_complete());
    assert_eq!(observed.load(Ordering::SeqCst), 0);
    pending.mark_complete();
    assert_eq!(combined.terminal_state(), Some(TaskState::Failed));
    assert_eq!(combined.panic_message().as_deref(), Some("first failure"));
    assert_eq!(observed.load(Ordering::SeqCst), 1);
    assert_eq!(pending.dependent_count(), 0);
}

#[test]
fn astra_m24_completion_racing_registration_never_loses_a_dependency() {
    const COUNT: usize = 1_000;
    let handles = (0..COUNT)
        .map(|_| JobHandle::pending_with_dependencies(0))
        .collect::<Vec<_>>();
    let worker_handles = handles.clone();
    let gate = Arc::new(std::sync::Barrier::new(2));
    let worker_gate = Arc::clone(&gate);
    let worker = std::thread::spawn(move || {
        worker_gate.wait();
        for (index, handle) in worker_handles.iter().enumerate() {
            match index % 3 {
                0 => handle.mark_complete(),
                1 => handle.mark_cancelled(),
                _ => handle.mark_panicked(Arc::from("racing failure")),
            }
        }
    });
    gate.wait();
    let combined = JobHandle::combine(&handles);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    combined.on_terminal(move || {
        sender.send(()).unwrap();
    });
    receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("every input must reach the combined fence");
    worker.join().unwrap();
    assert_eq!(combined.terminal_state(), Some(TaskState::Failed));
    assert_eq!(combined.panic_message().as_deref(), Some("racing failure"));
    assert!(handles.iter().all(JobHandle::is_complete));
    assert!(handles.iter().all(|handle| handle.dependent_count() == 0));
}

fn measure(count: usize, completed_percent: usize, optimized: bool) -> u128 {
    let inputs = (0..16)
        .map(|_| {
            (0..count)
                .map(|index| {
                    let handle = JobHandle::pending_with_dependencies(0);
                    if index * 100 < count * completed_percent {
                        handle.mark_complete();
                    }
                    handle
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut combined = Vec::with_capacity(inputs.len());
    let started = Instant::now();
    for handles in &inputs {
        combined.push(if optimized {
            JobHandle::combine(black_box(handles))
        } else {
            legacy_combine(black_box(handles))
        });
    }
    let elapsed = started.elapsed().as_nanos();
    for handles in &inputs {
        for handle in handles {
            handle.mark_complete();
        }
    }
    assert!(combined.iter().all(JobHandle::is_complete));
    black_box(combined);
    elapsed
}

fn percentiles(samples: &mut [u128]) -> [u128; 3] {
    samples.sort_unstable();
    [50, 95, 99].map(|p| samples[(samples.len() * p).div_ceil(100) - 1])
}

#[test]
#[ignore = "Windows release evidence through the coordinator"]
fn astra_m24_combined_fence_release_evidence() {
    assert!(
        !cfg!(debug_assertions),
        "performance evidence requires release"
    );
    const SAMPLES: usize = 101;
    for count in [0, 1, 1_000, 10_000] {
        for completed_percent in [0, 50, 100] {
            for _ in 0..8 {
                black_box(measure(count, completed_percent, false));
                black_box(measure(count, completed_percent, true));
            }
            let mut before = Vec::with_capacity(SAMPLES);
            let mut after = Vec::with_capacity(SAMPLES);
            for sample in 0..SAMPLES {
                for optimized in if sample % 2 == 0 {
                    [false, true]
                } else {
                    [true, false]
                } {
                    let elapsed = measure(count, completed_percent, optimized);
                    if optimized {
                        after.push(elapsed);
                    } else {
                        before.push(elapsed);
                    }
                }
            }
            let before = percentiles(&mut before);
            let after = percentiles(&mut after);
            let limit = if count >= 1_000 && completed_percent == 100 {
                80
            } else {
                105
            };
            let actual_completed = (0..count)
                .filter(|index| index * 100 < count * completed_percent)
                .count();
            println!("ASTRA_M24_COMBINED_FENCE profile=release inputs={count} requested_completed_percent={completed_percent} actual_completed={actual_completed} combines=16 samples={SAMPLES} warmup=8 before_p50_p95_p99_ns={before:?} after_p50_p95_p99_ns={after:?} p95_limit_percent={limit}");
            assert!(
                after[1] * 100 <= before[1] * limit,
                "combined fence p95 gate missed"
            );
        }
    }
}
