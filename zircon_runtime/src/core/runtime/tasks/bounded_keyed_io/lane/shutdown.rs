use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{LaneInner, LaneState};
use crate::core::runtime::tasks::bounded_keyed_io::{
    BoundedKeyedIoDiagnostics, BoundedKeyedIoShutdownReport,
};
use crate::core::runtime::tasks::JobHandle;

/// 关闭后的可查询所有权凭证；超时等待后仍可查看积压并重试等待。
/// 丢弃 guard 会无期限等待通道内的工作和任务句柄，调用方需先解除可能阻塞的外部依赖。
pub struct BoundedKeyedIoShutdownGuard {
    pub(super) lane: Arc<LaneInner>,
}

impl BoundedKeyedIoShutdownGuard {
    pub fn is_complete(&self) -> bool {
        shutdown_complete(&self.lane.lock())
    }

    /// Waits for every shutdown-pinned entry and its worker handle to finish.
    pub fn wait(&self) {
        let mut state = self.lane.lock();
        while !shutdown_complete(&state) {
            state = self
                .lane
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        let handles = state.active_handles.clone();
        drop(state);
        for handle in handles {
            handle.wait();
        }
    }

    /// 在统一截止时间前等待通道静止；返回 false 时 guard 仍保有清理权威。
    pub fn wait_until(&self, deadline: Instant) -> bool {
        let mut state = self.lane.lock();
        while !shutdown_complete(&state) {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            state = self
                .lane
                .changed
                .wait_timeout(state, deadline.saturating_duration_since(now))
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
        true
    }

    pub fn report(&self) -> BoundedKeyedIoShutdownReport {
        let state = self.lane.lock();
        BoundedKeyedIoShutdownReport {
            complete: shutdown_complete(&state),
            incomplete_entries: state.reserved_entries,
            failed: state.failed,
            cancelled: state.cancelled,
            diagnostics: diagnostics_for_state(&state),
        }
    }

    pub fn diagnostics(&self) -> BoundedKeyedIoDiagnostics {
        self.report().diagnostics
    }
}

impl Drop for BoundedKeyedIoShutdownGuard {
    fn drop(&mut self) {
        self.wait();
    }
}

pub(super) fn diagnostics_for_state(state: &LaneState) -> BoundedKeyedIoDiagnostics {
    let now = Instant::now();
    let oldest_age = oldest_age_at(
        now,
        state
            .queue
            .iter()
            .map(|entry| entry.enqueued_at)
            .chain(state.suspended.values().map(|entry| entry.enqueued_at))
            .chain(state.active.iter().map(|entry| entry.enqueued_at)),
    );
    BoundedKeyedIoDiagnostics {
        queue_entries: state.reserved_entries,
        retained_bytes: state.retained_bytes,
        in_flight: state.in_flight,
        oldest_age,
        submitted: state.submitted,
        completed: state.completed,
        failed: state.failed,
        cancelled: state.cancelled,
        superseded: state.superseded,
        coalesced: state.coalesced,
        worker_wall: state.worker_wall,
    }
}

fn oldest_age_at(now: Instant, enqueued_at: impl Iterator<Item = Instant>) -> Duration {
    enqueued_at
        .map(|enqueued_at| now.saturating_duration_since(enqueued_at))
        .max()
        .unwrap_or(Duration::ZERO)
}

fn shutdown_complete(state: &LaneState) -> bool {
    state.reserved_entries == 0
        && state.in_flight == 0
        && state.suspended.is_empty()
        && state.queue.is_empty()
        && state.active.is_none()
        && !state.pump_active
        && state.active_handles.iter().all(JobHandle::is_complete)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

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
        let source = include_str!("shutdown.rs");
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
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(65),
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
}
