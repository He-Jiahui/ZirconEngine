use std::hint::black_box;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;

use crate::core::runtime::tasks::{
    BoundedKeyedIoLane, BoundedKeyedIoLimits, JobScheduler, TaskPool, TaskPoolDescriptor,
};

use super::*;

const BENCHMARK_ENTRY_COUNT: usize = 4_096;
const BENCHMARK_SAMPLES: usize = 11;
const BENCHMARK_ITERATIONS: usize = 64;

#[test]
fn single_clock_diagnostics_preserves_oldest_age() {
    let now = Instant::now();
    let newest = now
        .checked_sub(Duration::from_millis(2))
        .expect("recent instant");
    let oldest = now
        .checked_sub(Duration::from_millis(17))
        .expect("recent instant");
    let middle = now
        .checked_sub(Duration::from_millis(8))
        .expect("recent instant");

    assert_eq!(
        oldest_age_at(now, [newest, oldest, middle].into_iter()),
        Duration::from_millis(17)
    );
    assert_eq!(oldest_age_at(now, std::iter::empty()), Duration::ZERO);
}

#[test]
fn single_clock_diagnostics_source_contract() {
    let source = include_str!("../shutdown.rs");
    let implementation = source
        .split_once("pub(super) fn diagnostics_for_state")
        .expect("diagnostics function")
        .1
        .split_once("\n}\n\nfn oldest_age_at")
        .expect("diagnostics function end")
        .0;

    assert_eq!(implementation.matches("Instant::now()").count(), 1);
    assert!(!implementation.contains(".elapsed()"));
    assert!(implementation.contains("let oldest_age = oldest_age_at("));
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_clock_diagnostics_release_benchmark() {
    let now = Instant::now();
    let enqueued_at = (0..BENCHMARK_ENTRY_COUNT)
        .map(|index| {
            now.checked_sub(Duration::from_micros(index as u64 + 1))
                .expect("recent benchmark instant")
        })
        .collect::<Vec<_>>();
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_age_scan(|| retired_oldest_age(&enqueued_at)));
            optimized_samples.push(measure_age_scan(|| {
                oldest_age_at(Instant::now(), enqueued_at.iter().copied())
            }));
        } else {
            optimized_samples.push(measure_age_scan(|| {
                oldest_age_at(Instant::now(), enqueued_at.iter().copied())
            }));
            retired_samples.push(measure_age_scan(|| retired_oldest_age(&enqueued_at)));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME59_SINGLE_CLOCK_DIAGNOSTICS_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
entries={BENCHMARK_ENTRY_COUNT} retired_clock_reads_per_snapshot=4096 \
optimized_clock_reads_per_snapshot=1 retired_p95_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(65),
        "single-clock diagnostics must reduce oldest-age scan P95 by at least 35%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

fn retired_oldest_age(enqueued_at: &[Instant]) -> Duration {
    enqueued_at
        .iter()
        .map(Instant::elapsed)
        .max()
        .unwrap_or(Duration::ZERO)
}

fn measure_age_scan(mut scan: impl FnMut() -> Duration) -> Duration {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        black_box(scan());
    }
    started.elapsed()
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}

#[test]
fn shutdown_poll_waits_for_real_worker_terminal_state() {
    let scheduler = shutdown_poll_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(1, 1), scheduler.clone());
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let handle = scheduler.schedule(move || {
        started_sender.send(()).unwrap();
        release_receiver.recv().unwrap();
    });
    let guard = lane.shutdown();
    // Declared after the guard so assertion unwinding releases the worker first.
    let mut release = ShutdownPollRelease(Some(release_sender));
    pin_shutdown_poll_worker(&guard, &handle);
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("real worker must start");

    assert!(guard.poll_result().is_none());
    release.release();
    wait_for_shutdown_poll_worker(&handle);
    let report = guard
        .poll_result()
        .expect("terminal worker")
        .expect("worker succeeded");
    assert!(report.complete);
    assert!(matches!(guard.poll_result(), Some(Ok(_))));
}

#[test]
fn shutdown_poll_retains_worker_failure_after_handle_reaping() {
    let scheduler = shutdown_poll_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(1, 1), scheduler.clone());
    let handle = scheduler.schedule(|| panic!("shutdown poll worker failure"));
    let guard = lane.shutdown();
    pin_shutdown_poll_worker(&guard, &handle);
    wait_for_shutdown_poll_worker(&handle);
    assert_eq!(handle.terminal_state(), Some(TaskState::Failed));
    {
        let mut state = guard.lane.lock();
        state.reap_terminal_handles();
        assert!(state.active_handles.is_empty());
    }
    for _ in 0..2 {
        assert!(matches!(guard.poll_result(), Some(Err(TaskState::Failed))));
    }
    assert!(catch_unwind(AssertUnwindSafe(|| drop(guard))).is_ok());
}

#[test]
fn shutdown_explicit_wait_preserves_panic_without_drop_rethrowing() {
    let scheduler = shutdown_poll_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(1, 1), scheduler.clone());
    let handle = scheduler.schedule(|| panic!("explicit shutdown wait failure"));
    let guard = lane.shutdown();
    pin_shutdown_poll_worker(&guard, &handle);
    wait_for_shutdown_poll_worker(&handle);
    assert!(catch_unwind(AssertUnwindSafe(|| guard.wait())).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| drop(guard))).is_ok());
}

fn shutdown_poll_scheduler() -> JobScheduler {
    JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ))
}

fn pin_shutdown_poll_worker(guard: &BoundedKeyedIoShutdownGuard, handle: &JobHandle) {
    guard.lane.lock().active_handles.push(handle.clone());
    let weak_lane = Arc::downgrade(&guard.lane);
    handle.on_terminal(move || {
        if let Some(lane) = weak_lane.upgrade() {
            let _state = lane.lock();
            lane.changed.notify_all();
        }
    });
}

fn wait_for_shutdown_poll_worker(handle: &JobHandle) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !handle.is_complete() {
        assert!(Instant::now() < deadline, "real worker did not terminalize");
        std::thread::sleep(Duration::from_millis(1));
    }
}

struct ShutdownPollRelease(Option<mpsc::Sender<()>>);

impl ShutdownPollRelease {
    fn release(&mut self) {
        self.0.take().expect("release once").send(()).unwrap();
    }
}

impl Drop for ShutdownPollRelease {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}
