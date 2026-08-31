use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 31;
const HANDOFFS_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260828iq_runtime289_tracy_borrows_before_profile_move() {
    let source = include_str!("../macros.rs");
    let macro_start = source
        .find("macro_rules! profile_dynamic_scope")
        .expect("dynamic profile macro");
    let macro_end = source[macro_start..]
        .find("macro_rules! profile_frame")
        .map(|offset| macro_start + offset)
        .expect("next profile macro");
    let implementation = &source[macro_start..macro_end];

    let tracy_span = implementation
        .find("let _zr_profile_dynamic_tracy_span")
        .expect("Tracy span");
    let profile_scope = implementation
        .rfind("let _zr_profile_dynamic_scope")
        .expect("combined profile scope");
    assert!(tracy_span < profile_scope);
    assert!(implementation.contains("_zr_profile_dynamic_scope_name,"));
    assert!(!implementation.contains("_zr_profile_dynamic_scope_name.clone()"));
}

#[test]
fn optimization_batch_20260828iq_runtime289_dynamic_name_remains_owned_for_both_sinks() {
    let source_name = "runtime.render_graph.dynamic.profile.name.".repeat(8);

    let (trace_length, profile_length) = candidate_handoff(source_name.as_str());

    assert_eq!(trace_length, source_name.len());
    assert_eq!(profile_length, source_name.len());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828iq_runtime289_profile_dynamic_name_handoff_bench() {
    let source_name = "runtime.render_graph.dynamic.profile.name.".repeat(16);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(source_name.as_str(), false));
            optimized_samples.push(measure(source_name.as_str(), true));
        } else {
            optimized_samples.push(measure(source_name.as_str(), true));
            legacy_samples.push(measure(source_name.as_str(), false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME289_PROFILE_DYNAMIC_NAME_HANDOFF_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
handoffs_per_sample={HANDOFFS_PER_SAMPLE} name_bytes={} \
legacy_name_allocations_per_handoff=2 optimized_name_allocations_per_handoff=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        source_name.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_handoff(source_name: &str) -> (usize, usize) {
    let trace_name = String::from(source_name);
    let trace_length = black_box(trace_name.as_str()).len();
    let profile_name = trace_name.clone();
    (trace_length, black_box(profile_name).len())
}

fn candidate_handoff(source_name: &str) -> (usize, usize) {
    let trace_name = String::from(source_name);
    let trace_length = black_box(trace_name.as_str()).len();
    let profile_name = trace_name;
    (trace_length, black_box(profile_name).len())
}

fn measure(source_name: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..HANDOFFS_PER_SAMPLE {
        let lengths = if optimized {
            candidate_handoff(black_box(source_name))
        } else {
            legacy_handoff(black_box(source_name))
        };
        checksum ^= black_box(lengths.0 ^ lengths.1);
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
