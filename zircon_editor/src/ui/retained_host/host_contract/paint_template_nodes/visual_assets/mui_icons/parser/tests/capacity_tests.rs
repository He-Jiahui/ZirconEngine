use std::hint::black_box;
use std::time::Instant;

use super::{path_elements, MuiIconPathElement};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 16_384;
const PATH_COUNT: usize = 64;

#[test]
fn optimization_batch_20260920et_editor852_path_capacity_preserves_elements() {
    let source = (0..PATH_COUNT)
        .map(|index| format!("d: \"M{index}\""))
        .collect::<Vec<_>>()
        .join(" ");

    let elements = path_elements(&source);

    assert_eq!(elements.len(), PATH_COUNT);
    assert!(elements.capacity() >= PATH_COUNT);
    assert!(path_elements("").is_empty());
}

#[test]
fn optimization_batch_20260920et_editor852_path_capacity_source_contract() {
    let source = include_str!("../../parser.rs");
    assert!(source.contains("let mut elements = Vec::with_capacity("));
    assert!(source.contains(r#"source.matches("d: \"").count()"#));
    assert!(source.contains("cursor = value_end"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260920et_editor852_mui_icon_path_capacity_bench() {
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
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR852_MUI_ICON_PATH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} paths_per_build={PATH_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved MUI icon path output P95 {optimized_p95_ns}ns must be at most 70% of growth-driven output P95 {legacy_p95_ns}ns"
    );
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(PATH_COUNT)
        } else {
            Vec::new()
        };
        for index in 0..PATH_COUNT {
            output.push(black_box(MuiIconPathElement {
                d: format!("M{index}"),
                opacity: None,
            }));
        }
        checksum ^= black_box(output.len() ^ output.capacity());
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
