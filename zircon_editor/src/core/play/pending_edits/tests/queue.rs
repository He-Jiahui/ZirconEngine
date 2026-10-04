use std::hint::black_box;

use super::*;

const PAGE_BUFFER_SAMPLE_PAIRS: usize = 17;
const PAGE_BUFFER_MEASURE_ITERATIONS: usize = 4_096;
const PAGE_BUFFER_SIZE: usize = 128;

fn elapsed_micros(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn legacy_page_buffer(values: &[usize]) -> Vec<usize> {
    values
        .iter()
        .filter(|_| true)
        .take(PAGE_BUFFER_SIZE)
        .copied()
        .collect()
}

fn reserved_page_buffer(values: &[usize]) -> Vec<usize> {
    let mut candidates = values.iter().filter(|_| true);
    let page_capacity = candidates
        .size_hint()
        .1
        .unwrap_or(PAGE_BUFFER_SIZE)
        .min(PAGE_BUFFER_SIZE);
    let mut page = Vec::with_capacity(page_capacity);
    for value in candidates.by_ref().take(PAGE_BUFFER_SIZE) {
        page.push(*value);
    }
    page
}

#[test]
fn removing_a_stale_location_fails_without_panicking() {
    let mut state = PendingEditQueueState::default();

    assert!(state.intent_at(PendingEditLocation::Pending(0)).is_none());
    assert!(state.intent_at_mut(PendingEditLocation::Retry(0)).is_none());
    assert!(state.remove(PendingEditLocation::Pending(0)).is_none());
    assert!(state.remove(PendingEditLocation::Retry(0)).is_none());
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826b_editor07_pending_edit_page_capacity_performance_evidence() {
    let values = (0..PAGE_BUFFER_SIZE).collect::<Vec<_>>();

    for _ in 0..4 {
        assert_eq!(
            black_box(legacy_page_buffer(&values)).len(),
            PAGE_BUFFER_SIZE
        );
        assert_eq!(
            black_box(reserved_page_buffer(&values)).len(),
            PAGE_BUFFER_SIZE
        );
    }

    let mut legacy_samples = Vec::with_capacity(PAGE_BUFFER_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(PAGE_BUFFER_SAMPLE_PAIRS);
    for sample_index in 0..PAGE_BUFFER_SAMPLE_PAIRS {
        let measure_legacy = || {
            elapsed_micros(|| {
                for _ in 0..PAGE_BUFFER_MEASURE_ITERATIONS {
                    black_box(legacy_page_buffer(black_box(&values)));
                }
            })
        };
        let measure_optimized = || {
            elapsed_micros(|| {
                for _ in 0..PAGE_BUFFER_MEASURE_ITERATIONS {
                    black_box(reserved_page_buffer(black_box(&values)));
                }
            })
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "EDITOR07_PENDING_EDIT_PAGE_CAPACITY_BENCH_V1 sample_pairs={} page_entries={} iterations_per_sample={} legacy_page_buffer_initial_capacity=0 optimized_page_buffer_initial_capacity={} legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        PAGE_BUFFER_SAMPLE_PAIRS,
        PAGE_BUFFER_SIZE,
        PAGE_BUFFER_MEASURE_ITERATIONS,
        PAGE_BUFFER_SIZE,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "reserved pending-edit page buffer p95 must be at least 20% below progressive growth: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}
