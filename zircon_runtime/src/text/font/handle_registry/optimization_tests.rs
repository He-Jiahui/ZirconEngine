use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const TEST_COLLECTION: TextFontCollectionHandle = TextFontCollectionHandle::new(201);

#[test]
fn runtime80_handle_projection_preserves_stale_and_missing_results() {
    let generation = 7;
    let current = TextFontFaceHandle::new(TEST_COLLECTION, 1, generation);
    let stale = TextFontFaceHandle::new(TEST_COLLECTION, 2, generation + 1);
    let pairs = [(Some(current), None), (Some(stale), None), (None, None)];
    let resolved = HashMap::from([((Some(current), None), (Some(FontFaceId(11)), None))]);

    let (result, rejected) = project_resolved_pairs(&pairs, TEST_COLLECTION, generation, &resolved);

    assert_eq!(
        result,
        vec![(Some(FontFaceId(11)), None), (None, None), (None, None)]
    );
    assert_eq!(rejected, 1);
}

#[test]
#[ignore = "release performance evidence"]
fn runtime80_handle_allocation_free_normalization_benchmark_evidence() {
    const PAIRS: usize = 16_384;
    let generation = 9;
    let pairs = handle_pairs(PAIRS, generation);
    let mut legacy = || {
        let normalized = pairs
            .iter()
            .map(|pair| normalize_text_pair(*pair, TEST_COLLECTION, generation))
            .collect::<Vec<_>>();
        unique_text_pairs(&normalized).len()
    };
    let mut optimized =
        || unique_current_text_pairs(black_box(&pairs), TEST_COLLECTION, generation).len();
    let expected = PAIRS / 2;
    assert_eq!(legacy(), expected);
    assert_eq!(optimized(), expected);
    benchmark_pair(
        &mut legacy,
        &mut optimized,
        expected,
        "RUNTIME80_ALLOCATION_FREE_HANDLE_NORMALIZATION_BENCH_V1",
        "legacy_normalized_entries=16384 optimized_normalized_entries=0",
        4,
        3,
    );
}

#[test]
#[ignore = "release performance evidence"]
fn runtime80_handle_single_pass_rejection_projection_benchmark_evidence() {
    const PAIRS: usize = 16_384;
    let generation = 13;
    let pairs = handle_pairs(PAIRS, generation);
    let resolved = pairs
        .iter()
        .copied()
        .map(|pair| {
            (
                pair,
                (pair.0.map(|handle| FontFaceId(handle.index as u64)), None),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut legacy = || {
        let result = pairs
            .iter()
            .map(|pair| resolved.get(pair).copied().unwrap_or((None, None)))
            .collect::<Vec<_>>();
        let rejected = pairs
            .iter()
            .zip(&result)
            .filter(|((face, instance), (resolved_face, resolved_instance))| {
                (face.is_some() && resolved_face.is_none())
                    || (instance.is_some() && resolved_instance.is_none())
            })
            .count();
        result.len() + rejected
    };
    let mut optimized = || {
        let (result, rejected) =
            project_resolved_pairs(black_box(&pairs), TEST_COLLECTION, generation, &resolved);
        result.len() + rejected
    };
    assert_eq!(legacy(), PAIRS);
    assert_eq!(optimized(), PAIRS);
    benchmark_pair(
        &mut legacy,
        &mut optimized,
        PAIRS,
        "RUNTIME80_SINGLE_PASS_HANDLE_REJECTION_PROJECTION_BENCH_V1",
        "legacy_projection_passes=2 optimized_projection_passes=1",
        10,
        9,
    );
}

fn handle_pairs(count: usize, generation: u64) -> Vec<TextFontHandlePair> {
    (0..count)
        .map(|index| {
            let handle = TextFontFaceHandle::new(TEST_COLLECTION, (index / 2) as u32, generation);
            (Some(handle), None)
        })
        .collect()
}

fn benchmark_pair(
    legacy: &mut impl FnMut() -> usize,
    optimized: &mut impl FnMut() -> usize,
    expected: usize,
    marker: &str,
    structure: &str,
    optimized_multiplier: u128,
    legacy_multiplier: u128,
) {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for index in 0..SAMPLE_PAIRS {
        let measure = |operation: &mut dyn FnMut() -> usize| {
            let started = Instant::now();
            assert_eq!(black_box(operation()), expected);
            started.elapsed().as_nanos()
        };
        if index % 2 == 0 {
            legacy_ns.push(measure(legacy));
            optimized_ns.push(measure(optimized));
        } else {
            optimized_ns.push(measure(optimized));
            legacy_ns.push(measure(legacy));
        }
    }
    let legacy_p50 = percentile(&legacy_ns, 50);
    let legacy_p95 = percentile(&legacy_ns, 95);
    let optimized_p50 = percentile(&optimized_ns, 50);
    let optimized_p95 = percentile(&optimized_ns, 95);
    assert!(
        optimized_p95.saturating_mul(optimized_multiplier)
            <= legacy_p95.saturating_mul(legacy_multiplier),
        "performance gate failed: legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
    println!(
        "{marker} pairs=16384 sample_pairs={SAMPLE_PAIRS} {structure} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns)
    );
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
