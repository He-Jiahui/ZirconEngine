use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use super::finalize_captured_output_tail;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 3_000;
const TAIL_BYTES: usize = 8 * 1024;

#[test]
fn optimization_batch_20260829ak_editor256_tail_finalization_preserves_wrapped_order() {
    let mut tail = VecDeque::with_capacity(16);
    tail.extend(0_u8..12);
    tail.drain(..5);
    tail.extend(12_u8..20);
    let expected = tail.iter().copied().collect::<Vec<_>>();

    assert_eq!(finalize_captured_output_tail(tail), expected);
}

#[test]
fn optimization_batch_20260829ak_editor256_capture_consumes_the_existing_tail_buffer() {
    let source = include_str!("../compile_host.rs");
    let capture = source
        .split("fn capture_output_stream")
        .nth(1)
        .expect("command output capture")
        .split("fn join_output_capture")
        .next()
        .expect("command output capture body");

    assert!(capture.contains("finalize_captured_output_tail(tail)"));
    assert!(capture.contains("Vec::from(tail)"));
    assert!(!capture.contains("tail.into_iter().collect()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ak_editor256_zero_copy_output_tail_finalization_bench() {
    let template = tail_template();
    assert_eq!(
        optimized_tail(template.clone()),
        legacy_tail(template.clone())
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &template));
            optimized_samples.push(measure(true, &template));
        } else {
            optimized_samples.push(measure(true, &template));
            legacy_samples.push(measure(false, &template));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR256_ZERO_COPY_COMMAND_OUTPUT_TAIL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} tail_bytes_per_build={TAIL_BYTES} \
legacy_tail_allocations_per_build=2 optimized_tail_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn tail_template() -> VecDeque<u8> {
    let mut tail = VecDeque::with_capacity(TAIL_BYTES);
    tail.extend((0..TAIL_BYTES).map(|index| (index % 251) as u8));
    let wrap_bytes = TAIL_BYTES / 4;
    tail.drain(..wrap_bytes);
    tail.extend((0..wrap_bytes).map(|index| (index % 239) as u8));
    tail
}

fn legacy_tail(tail: VecDeque<u8>) -> Vec<u8> {
    tail.into_iter().collect()
}

fn optimized_tail(tail: VecDeque<u8>) -> Vec<u8> {
    finalize_captured_output_tail(tail)
}

fn measure(optimized: bool, template: &VecDeque<u8>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let tail = black_box(template).clone();
        let bytes = if optimized {
            optimized_tail(tail)
        } else {
            legacy_tail(tail)
        };
        checksum = checksum.wrapping_add(black_box(bytes).len());
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
