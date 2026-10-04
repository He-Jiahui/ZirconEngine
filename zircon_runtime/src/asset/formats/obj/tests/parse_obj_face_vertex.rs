use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_20260831gq_runtime572_component_compatibility() {
    assert_eq!(obj_face_vertex_components("7"), ("7", None, None));
    assert_eq!(obj_face_vertex_components("7/3"), ("7", Some("3"), None));
    assert_eq!(
        obj_face_vertex_components("7//2"),
        ("7", Some(""), Some("2"))
    );
    assert_eq!(
        obj_face_vertex_components("7/3/2"),
        ("7", Some("3"), Some("2"))
    );
    assert_eq!(
        obj_face_vertex_components("7/3/2/ignored"),
        ("7", Some("3"), Some("2"))
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260831gq_runtime572_obj_face_component_single_scan_benchmark() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 250_000;
    const TOKENS: [&str; 4] = ["1845/29/77", "-2//-1", "42/7", "11"];

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut checksum = 0usize;
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            let (elapsed, value) = measure(ITERATIONS, &TOKENS, legacy_component_lengths);
            legacy_ns.push(elapsed);
            checksum ^= value;
            let (elapsed, value) = measure(ITERATIONS, &TOKENS, optimized_component_lengths);
            optimized_ns.push(elapsed);
            checksum ^= value;
        } else {
            let (elapsed, value) = measure(ITERATIONS, &TOKENS, optimized_component_lengths);
            optimized_ns.push(elapsed);
            checksum ^= value;
            let (elapsed, value) = measure(ITERATIONS, &TOKENS, legacy_component_lengths);
            legacy_ns.push(elapsed);
            checksum ^= value;
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(95),
        "single-scan P95 must be at least 5% below split traversal: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "RUNTIME572_OBJ_FACE_COMPONENT_SINGLE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} tokens_per_iteration={} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        TOKENS.len(),
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );

    fn measure(iterations: usize, tokens: &[&str], operation: fn(&str) -> usize) -> (u128, usize) {
        let started = Instant::now();
        let mut checksum = 0usize;
        for index in 0..iterations {
            checksum = checksum.wrapping_add(operation(black_box(tokens[index % tokens.len()])));
        }
        (started.elapsed().as_nanos(), black_box(checksum))
    }
}

fn legacy_component_lengths(token: &str) -> usize {
    let mut parts = token.split('/');
    parts.next().unwrap_or_default().len()
        + parts.next().map_or(0, str::len)
        + parts.next().map_or(0, str::len)
}

fn optimized_component_lengths(token: &str) -> usize {
    let (position, uv, normal) = obj_face_vertex_components(token);
    position.len() + uv.map_or(0, str::len) + normal.map_or(0, str::len)
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100).max(1);
    ordered[rank - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
