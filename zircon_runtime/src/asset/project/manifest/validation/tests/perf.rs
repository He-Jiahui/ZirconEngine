use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::project::MAX_PROJECT_ASSET_ROOTS;

const ROOT_COUNT: usize = MAX_PROJECT_ASSET_ROOTS;
const SAMPLE_COUNT: usize = 31;

fn legacy_membership(roots: &[String]) -> usize {
    let mut seen = HashSet::new();
    roots
        .iter()
        .filter(|root| seen.insert(root.as_str()))
        .count()
}

fn optimized_membership(roots: &[String]) -> usize {
    let mut seen = HashSet::with_capacity(roots.len());
    roots
        .iter()
        .filter(|root| seen.insert(root.as_str()))
        .count()
}

#[test]
fn optimization_batch_ir_runtime629_manifest_membership_reserves_input_bounds() {
    let source = include_str!("../../validation.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("manifest validation production source");

    assert!(production.contains("HashMap::with_capacity(self.asset_roots.len())"));
    assert!(production.contains("HashSet::with_capacity(self.ui_roots.len())"));
    assert!(!production.contains("let mut roots = HashSet::new()"));
    assert!(!production.contains("let mut ui_roots = HashSet::new()"));
}

#[test]
#[ignore = "Windows Release helper microbenchmark; real caller evidence is required"]
fn optimization_batch_ir_runtime629_manifest_membership_performance_evidence() {
    let roots = (0..ROOT_COUNT)
        .map(|index| format!("generated/project/assets/root_{index:05}/nested"))
        .collect::<Vec<_>>();
    assert_eq!(legacy_membership(&roots), optimized_membership(&roots));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_membership(black_box(&roots)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_membership(black_box(&roots)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_membership(black_box(&roots)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_membership(black_box(&roots)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let legacy_p99 = nearest_rank(&legacy_samples, 99);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let optimized_p99 = nearest_rank(&optimized_samples, 99);
    println!(
        "RUNTIME629_PREALLOCATED_MANIFEST_ROOT_MEMBERSHIP_BENCH_V2 \
         roots={ROOT_COUNT} samples={SAMPLE_COUNT} benchmark_scope=microbenchmark \
         real_caller_required=true percentile_method=nearest_rank \
         legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} \
         optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} \
         optimized_p99_ns={optimized_p99} target_ratio_bp=8500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated manifest membership P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}
