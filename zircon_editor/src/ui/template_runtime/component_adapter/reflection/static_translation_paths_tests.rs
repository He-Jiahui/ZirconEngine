use std::hint::black_box;
use std::time::Instant;

use super::TRANSLATION_INSPECTOR_PATHS;

const SAMPLE_PAIRS: usize = 31;
const VECTORS_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829v_editor241_translation_paths_preserve_axis_order() {
    assert_eq!(
        TRANSLATION_INSPECTOR_PATHS,
        [
            "transform.translation.x",
            "transform.translation.y",
            "transform.translation.z",
        ]
    );
}

#[test]
fn optimization_batch_20260829v_editor241_translation_projection_uses_static_paths() {
    let source = include_str!("../reflection.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn apply_translation_vector")
        .nth(1)
        .and_then(|body| body.split("fn reflection_result").next())
        .expect("translation projection");

    assert!(body.contains("TRANSLATION_INSPECTOR_PATHS"));
    assert!(body.contains("zip(translation.iter())"));
    assert!(!body.contains("format!(\"transform.translation.{axis}\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829v_editor241_static_translation_inspector_paths_bench() {
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
        "EDITOR241_STATIC_TRANSLATION_INSPECTOR_PATHS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
vectors_per_sample={VECTORS_PER_SAMPLE} axes_per_vector=3 \
legacy_path_allocations_per_vector=3 optimized_path_allocations_per_vector=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_path_bytes() -> usize {
    ["x", "y", "z"]
        .into_iter()
        .map(|axis| format!("transform.translation.{axis}"))
        .map(|path| black_box(path).len())
        .sum()
}

fn optimized_path_bytes() -> usize {
    black_box(TRANSLATION_INSPECTOR_PATHS)
        .iter()
        .map(|path| black_box(path.len()))
        .sum()
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..VECTORS_PER_SAMPLE {
        checksum = checksum.wrapping_add(if optimized {
            optimized_path_bytes()
        } else {
            legacy_path_bytes()
        });
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
