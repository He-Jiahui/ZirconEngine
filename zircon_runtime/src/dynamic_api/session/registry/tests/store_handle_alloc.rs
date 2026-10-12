use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::{SessionRegistry, SessionRegistryInsertError};

const PERFORMANCE_SAMPLE_PAIRS: usize = 21;
const PERFORMANCE_ITERATIONS: usize = 250_000;
const BASIS_POINTS_SCALE: u128 = 10_000;
const PERFORMANCE_MAX_RATIO_BPS: u128 = 7_500;

fn nearest_rank_percentile(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn measure_legacy_atomic_allocation() -> Duration {
    let next_handle = AtomicU64::new(1);
    let started = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        black_box(next_handle.fetch_add(1, Ordering::SeqCst));
    }
    started.elapsed()
}

fn measure_checked_allocation() -> Duration {
    let mut registry = SessionRegistry::default();
    let started = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        black_box(registry.try_allocate_handle().unwrap());
    }
    started.elapsed()
}

#[test]
fn session_handle_allocation_exhausts_after_the_maximum_value_without_wrapping() {
    let mut registry = SessionRegistry::default();
    registry.next_handle = u64::MAX;

    assert_eq!(registry.try_allocate_handle().unwrap(), u64::MAX);
    assert_eq!(
        registry.try_allocate_handle(),
        Err(SessionRegistryInsertError::HandleSpaceExhausted)
    );
    assert_eq!(
        registry.try_allocate_handle(),
        Err(SessionRegistryInsertError::HandleSpaceExhausted)
    );
}

#[test]
fn session_handle_allocation_rejects_zero_as_exhausted() {
    let mut registry = SessionRegistry::default();
    registry.next_handle = 0;

    assert_eq!(
        registry.try_allocate_handle(),
        Err(SessionRegistryInsertError::HandleSpaceExhausted)
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn checked_session_handle_allocation_release_performance_acceptance() {
    black_box(measure_legacy_atomic_allocation());
    black_box(measure_checked_allocation());

    let mut legacy_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_PAIRS);
    let mut checked_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_PAIRS);
    for pair in 0..PERFORMANCE_SAMPLE_PAIRS {
        let (legacy, checked) = if pair % 2 == 0 {
            (
                measure_legacy_atomic_allocation(),
                measure_checked_allocation(),
            )
        } else {
            let checked = measure_checked_allocation();
            let legacy = measure_legacy_atomic_allocation();
            (legacy, checked)
        };
        legacy_samples.push(legacy);
        checked_samples.push(checked);
    }

    let legacy_p95 = nearest_rank_percentile(&legacy_samples, 95);
    let checked_p95 = nearest_rank_percentile(&checked_samples, 95);
    let ratio_bps = checked_p95.as_nanos() * BASIS_POINTS_SCALE / legacy_p95.as_nanos().max(1);
    let legacy_samples_ns = legacy_samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let checked_samples_ns = checked_samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "PERF-MVP-INTERFACE01-HANDLE: sample_pairs={PERFORMANCE_SAMPLE_PAIRS};order=alternating_legacy_first_even;iterations={PERFORMANCE_ITERATIONS};legacy_samples_ns={legacy_samples_ns};optimized_samples_ns={checked_samples_ns};legacy_p95_ns={};optimized_p95_ns={};ratio_bps={ratio_bps};threshold_bps={PERFORMANCE_MAX_RATIO_BPS}",
        legacy_p95.as_nanos(),
        checked_p95.as_nanos(),
    );

    assert!(
        checked_p95.as_nanos() * BASIS_POINTS_SCALE
            <= legacy_p95.as_nanos() * PERFORMANCE_MAX_RATIO_BPS,
        "checked handle allocation P95 {checked_p95:?} must be at least 25% faster than legacy SeqCst allocation P95 {legacy_p95:?}"
    );
}
