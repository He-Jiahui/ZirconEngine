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
