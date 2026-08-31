use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 10_000;
const FLOATING_WINDOW_COUNT: usize = 512;

#[test]
fn optimization_batch_20260829am_editor258_preallocated_targets_preserve_filter_order() {
    let legacy = legacy_native_targets(FLOATING_WINDOW_COUNT);
    let optimized = optimized_native_targets(FLOATING_WINDOW_COUNT);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.capacity(), FLOATING_WINDOW_COUNT);
    assert_eq!(optimized_native_targets(0).capacity(), 0);
}

#[test]
fn optimization_batch_20260829am_editor258_native_target_path_preallocates_from_window_count() {
    let source = include_str!("../target.rs");
    let builder = source
        .split("fn collect_native_floating_window_targets")
        .nth(1)
        .expect("native floating window target builder")
        .split("#[cfg(test)]")
        .next()
        .expect("native floating window target builder body");

    let reserve = builder
        .find("targets.reserve_exact(model.floating_windows.len());")
        .expect("native target upper-bound reservation");
    let push = builder.find("targets.push(").expect("native target append");
    assert!(builder.contains("if targets.is_empty()"));
    assert!(reserve < push);
    assert!(!builder.contains(".filter_map("));
    assert!(!builder.contains(".collect()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829am_editor258_preallocated_native_window_targets_bench() {
    assert_eq!(
        optimized_native_targets(FLOATING_WINDOW_COUNT),
        legacy_native_targets(FLOATING_WINDOW_COUNT)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR258_PREALLOCATED_NATIVE_WINDOW_TARGETS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} floating_windows_per_build={FLOATING_WINDOW_COUNT} \
legacy_vector_allocations_per_build=8 optimized_vector_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_native_targets(window_count: usize) -> Vec<usize> {
    (0..window_count)
        .filter_map(|index| (index % 8 != 7).then_some(index))
        .collect()
}

fn optimized_native_targets(window_count: usize) -> Vec<usize> {
    let mut targets = Vec::new();
    for index in 0..window_count {
        if index % 8 != 7 {
            if targets.is_empty() {
                targets.reserve_exact(window_count);
            }
            targets.push(index);
        }
    }
    targets
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let targets = if optimized {
            optimized_native_targets(black_box(FLOATING_WINDOW_COUNT))
        } else {
            legacy_native_targets(black_box(FLOATING_WINDOW_COUNT))
        };
        checksum = checksum.wrapping_add(black_box(targets).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
