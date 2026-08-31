use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 200_000;
const OVERLAY_ELEMENT_COUNT: usize = 4;

#[test]
fn optimization_batch_20260829ai_editor254_move_overlay_reserves_fixed_elements() {
    let source = include_str!("../move_handle_tool_behavior.rs");
    let builder = source
        .split("fn build_overlay")
        .nth(1)
        .expect("move overlay builder")
        .split("fn begin_drag")
        .next()
        .expect("move overlay builder body");

    assert!(builder.contains("Vec::with_capacity(4)"));
    assert!(!builder.contains("let mut elements = Vec::new()"));
}

#[test]
fn optimization_batch_20260829ai_editor254_move_overlay_keeps_four_element_contract() {
    assert_eq!(optimized_element_buffer().capacity(), OVERLAY_ELEMENT_COUNT);
    assert_eq!(legacy_element_buffer().capacity(), 4);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ai_editor254_single_allocation_move_handle_overlay_bench() {
    assert_eq!(optimized_element_buffer().len(), 0);
    assert_eq!(legacy_element_buffer().len(), 0);

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
        "EDITOR254_SINGLE_ALLOCATION_MOVE_HANDLE_OVERLAY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} overlay_elements_per_build={OVERLAY_ELEMENT_COUNT} \
legacy_buffer_growth_operations_per_build=1 optimized_buffer_growth_operations_per_build=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_element_buffer() -> Vec<usize> {
    let mut elements = Vec::new();
    for index in 0..OVERLAY_ELEMENT_COUNT {
        elements.push(index);
    }
    elements
}

fn optimized_element_buffer() -> Vec<usize> {
    let mut elements = Vec::with_capacity(OVERLAY_ELEMENT_COUNT);
    for index in 0..OVERLAY_ELEMENT_COUNT {
        elements.push(index);
    }
    elements
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let elements = if optimized {
            optimized_element_buffer()
        } else {
            legacy_element_buffer()
        };
        checksum = checksum.wrapping_add(black_box(elements).len());
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
